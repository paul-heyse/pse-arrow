---
id: ADR-0229
title: Own product declarations, deployment and production profiles
status: proposed
date: 2026-10-26
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-07, DP-01, DP-04, DP-09]
blueprint: [§D1, §3.2, §20.6, §21.1]
review: docs/design_review/reviews/design_review_product-and-workload-ownership_2026-10-26.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A product operation requires development storage or a deployment consumer must reinterpret allocation policy.
verification: Plan56e product Cargo closure and generation isolation controls, frozen import probe, shared allocation materialization and PID reuse/borrower drain controls.
standard: core-3.4/process-simulator-1.5
scenarios: [docs/plans/56e-product-deployment-and-resource-ownership.md#acceptance]
---

# ADR-0229: Own product declarations, deployment and production profiles

## Context

DA-019/020/036/037 expose development storage in product compilation, mirrored deployment meaning,
conformance-owned production settings and cyclic process/borrower knowledge. The maintainer selected
Plan56 RC02/RC03 and the remaining RC04/RC05 proposals before authorizing joint 56d/56e execution.

## Scope

Amend blueprint §D1/§3.2/§20.6/§21.1 for E01–E04 and the necessary F01 generation prerequisites.
Scientific meanings remain registry-owned. This changes declaration responsibility, product roots
and allocation authority; it does not introduce a crate, engine, numerical method or host limit.
Formal decision acceptance remains separate from authorized implementation.

## Drivers

A private evidence vocabulary change must not invalidate scientific contracts or require product
consumers to compile development analytics. Product receivers need complete immutable executable
imports and one validated allocation, without requiring fixture/report policy to observe borrowers.
Process observations must preserve uncertainty and leave liveness decisions with their owner.

## Options

Keeping one catalogue with universally enabled features preserves the dependency cause. A new
service/crate creates another deployment and interface without a separate distribution need.
Select owner-local Rust serde document sets and non-scientific enums, published by the existing
generator's output families. Keep canonical/scientific declarations in the registry and compose
development contributions only in xtask. Move the complete private evidence/knowledge backend there.
A whole tooling-module move would incorrectly move product observability and scientific IPC work.

Python-only allocation validation would retain mirrors. Select Rust-owned versioned deployment,
profile, context, receiver and readiness documents and one relational validator; generated Python
and schemas support host realization. Freezing every script causes unrelated invalidation; select
AST-derived owned imports, package parents, required generated/configuration/executable inputs and
explicit dynamic imports, verified by isolated frozen import before generation publication.

## Outcome

Production worker/project binaries live in existing pse-runtime; recovery remains in pse-operations,
with fixtures and exploratory native lanes in xtask. Product IPC/contained-copy primitives move to a
narrow shared product owner beneath portability and diagnostic consumers. Private storage exposes
no raw canonical scientific client. Owner-local types lower through existing generation, without a
parallel generator or private vocabulary in foundation outputs.

Production execution profiles live in separately typed sections of .config/agent-capacity.toml;
conformance selects a profile. Rust materializes and validates allocation once; host admission,
supervision and native workers consume it. Preserve every selected profile value, explicit override
precedence, reference sixteen lanes, distinct fixture lanes, physical/logical CPU contracts and
uncapped ordinary workstation behavior. Host, observer, receiver and worker scopes remain distinct.

A dependency-leaf process observer reports authenticated facts and uncertainty. Existing resource
admission owns borrower discovery; supervision and report/fixture owners consume its narrow
interface. Launcher death does not prove descendant drain, and unknown authority remains protective.
No second ledger, universal alive predicate or native file-authority replacement is introduced.

### Consequences

Declarations, generated families, Cargo consumers and product entry points must migrate together.
Remove obsolete mirrors and forced evidence-query product activation after consumers move. Generation
identity includes all required product inputs while excluding unrelated development scripts.

### Compensating controls

Test private-only declaration changes against scientific output stability and actual normal Cargo
closure. Exercise frozen missing/generated/dynamic imports, relational refusal before effects,
profile propagation, PID reuse, uncertain owners, surviving descendants and repeated custody drain.

### Confirmation

Proposed before dependent implementation at c5ae90b84. The focused review assesses the target;
56e owns actual delivery and qualification. No build-time or performance benefit is yet measured.

## Pros and cons

Responsibility-local changes become possible without a new distribution boundary. Integration must
migrate generated roots and all allocation consumers coherently; type generation alone cannot prove
that the supported product feature closure is clean.

## More information

[Plan56e](../plans/56e-product-deployment-and-resource-ownership.md) owns E01–E05;
[Plan56](../plans/56-design-principle-alignment.md) owns rule and finding dispositions.
ADR-0116's central-enum requirement is amended for non-scientific owner-local documents only.
ADR-0228's file authority and product recovery root remain intact.

## Status history

- 2026-10-26: Proposed after explicit RC02/RC03 and remaining RC04/RC05 selection and execution authorization.
