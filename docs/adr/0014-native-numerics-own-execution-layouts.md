---
id: ADR-0014
title: Adopt D11: execution layouts are compiled artifacts and zero-copy is a preference
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-36, DM-37, DM-39, DM-59]
blueprint: [§D11, §18.2, §15.4]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: ADR-0047
revisit: A borrow precondition is relaxed, or a measurement shows the copy path dominates an evaluation benchmark
verification: `tests/engine` borrow-precondition tests; the §24.3 evaluation-program benchmarks

---

# ADR-0014: Adopt D11: execution layouts are compiled artifacts and zero-copy is a preference

## Context

Blueprint D11 makes the evaluation program, sparse Jacobian structures and solver workspaces compiled artifacts that borrow Arrow buffers when layouts permit and copy when they do not. The second review's R2-6 corrected stale claims about `faer`, `petgraph` and `object_store` surfaces at the pinned versions.

## Scope

Binds the ownership rules at the Arrow-to-native boundary and the numerical routes the diagnostics use. It does not bind the solver interface (§18.3, ADR-0028).

## Drivers

Charter §G lists "it is zero-copy" as a claim to check; a sliced, filtered or nullable array cannot be borrowed; a claimed numerical route that no library provides is a capability lie (DM-43).

## Options

Borrow unconditionally — rejected: a sliced or nullable array would be read past its validity. Claim an iterative SVD without a route — rejected by F17; the route is now shift-invert over `faer`'s `matrix_free` with a dense limit.

## Outcome

Arrow buffers are borrowed only when the array is contiguous (`ptr_offset() == 0`, one chunk) and null-free; otherwise the data is copied through a checked slice. The workspace is native. Every numerical route names the library function it uses.

### Consequences

The boundary code has two paths and both must be tested; the condition-number default becomes a Hager-Higham estimate with the exact Frobenius form opt-in.

### Compensating controls

Borrow preconditions are asserted, not assumed; `tests/engine` constructs sliced, filtered and nullable arrays and asserts the copy path is taken.

### Confirmation

`rust / test` runs the engine layer; the §24.3 benchmarks record the copy rate under recorded conditions so the claim stays Measured rather than asserted.

## Pros and cons

Two paths cost code; one path costs correctness.

## More information

Blueprint §D11, §18.2 (native evaluation program), §15.4-§15.5 (numerical diagnostics); review finding R2-6.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-13 — superseded by ADR-0047.
