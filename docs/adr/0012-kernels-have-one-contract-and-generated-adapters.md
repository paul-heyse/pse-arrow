---
id: ADR-0012
title: Adopt D9: every constitutive computation is a KernelSpec with generated bindings
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-19, DM-25, DM-44, DM-52]
blueprint: [§D9, §18.5, §21.4]
review: not-required: neither review raised a finding against D9
evidence: Proposed
supersedes: []
superseded-by: ADR-0043
revisit: A kernel needs a binding form the six generated ones cannot express, or a third-party compiled kernel package is admitted (§22, ADR-0037)
verification: `tests/conformance` kernel-adapter equivalence tests; `rust / codegen-diff`

---

# ADR-0012: Adopt D9: every constitutive computation is a KernelSpec with generated bindings

## Context

Blueprint D9 gives every code-implemented constitutive computation a `KernelSpec` with identity, signature in physical types, mathematical behaviour, derivative availability, execution forms, failure behaviour and backend bindings, from which scalar, batched Arrow, derivative, DataFusion UDF, NL external-function and Pyomo bindings are generated.

## Scope

Binds the kernel contract and the generated binding set. Which kernels exist is §9 and the reference packages.

## Drivers

Six hand-written bindings per kernel is five too many; a kernel whose derivative availability is undeclared forces the Hessian profile to limited memory silently.

## Options

Hand-written adapters per backend — rejected: they drift, and the NL and Pyomo forms must agree bit for bit with the native one. A trait per backend — rejected: the spec is data (D1), so the adapters are generated from it.

## Outcome

One `KernelSpec` per computation; all six binding forms are generated; derivative availability and failure behaviour are declared, not inferred.

### Consequences

Adding a kernel is a spec plus a Rust body plus tests; the adapters come from the generator and are committed and diffed (ADR-0031).

### Compensating controls

Kernel-adapter equivalence tests compare scalar, batched and generated-UDF evaluation on the same inputs; `KernelSpec` digests enter the plan fingerprint (ADR-0019).

### Confirmation

`rust / test` runs the equivalence layer; `rust / codegen-diff` fails if a generated adapter is edited by hand.

## Pros and cons

The spec is verbose; it is also the only place a kernel's derivative and failure contract is written down.

## More information

Blueprint §D9, §18.5 (generated kernel adapters), §21.4 (opaque kernels and ASL), §22.3.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-13 — superseded by ADR-0043.
