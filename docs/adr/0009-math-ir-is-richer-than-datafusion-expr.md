---
id: ADR-0009
title: Adopt D6: the math IR is relations, and DataFusion Expr computes over it
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-21, DM-22, DM-10]
blueprint: [§D6, §7.1, §7.2]
review: not-required: neither review raised a finding against D6
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A backend requires an operator the IR cannot express without scalarizing earlier than §7.1 allows
verification: `tests/conformance` operator-contract tests; the expression DSL round-trip test at the phase-0 exit

---

# ADR-0009: Adopt D6: the math IR is relations, and DataFusion Expr computes over it

## Context

Blueprint D6 keeps expression nodes, arguments and typed operator payloads as relations, and keeps indexed operators (`SumOver`, `Gather`, `Broadcast`, `Derivative`, `Integral`, `ImplicitSystem`, `KernelCall`) alive until a backend requires scalarization.

## Scope

Binds the representation of mathematics. It does not bind the operator catalog's contents (§7.2) or canonicalization (§7.4).

## Drivers

DataFusion `Expr` has no indexed reduction, no derivative and no kernel call with declared derivative availability; scalarizing early destroys the structure discretization and index expansion need.

## Options

Use `Expr` as the IR — rejected: the seven indexed operators have no representation and the IR would be tied to the engine version. A hand-written Rust enum — rejected by D1.

## Outcome

Expression nodes, arguments and typed payloads are relations; DataFusion `Expr` is used to compute over the IR, never to be it.

### Consequences

Every operator needs a contract record (§7.3) and a lowering per backend; adding one is a code change by §22.3's own table.

### Compensating controls

Canonicalization (P10) is hand-written and complete on its own; `egglog` stays optional and phase 4 (ADR-0037).

### Confirmation

Operator-contract conformance tests run per backend; the DSL round-trip is a phase-0 exit criterion (§25).

## Pros and cons

Keeping indexed operators alive costs a scalarization pass (P12); folding them early costs the ability to discretize.

## More information

Blueprint §D6, §7.1 (design constraints), §7.2 (operator catalog), §14.1 (P12).

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
