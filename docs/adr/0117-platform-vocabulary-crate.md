---
id: ADR-0117
title: Hold the platform vocabulary enums in a crate beneath the registry and the generated model
status: accepted
date: 2026-09-28
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-04, DP-01, DP-17]
blueprint: [§3.2, §4.2]
review: docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md#addendum-adr-0117
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A platform vocabulary needs behaviour that depends on pse-schema's Arrow model, or the generator ceiling is redefined so that pse-codegen may depend on generated crates.
verification: Plan 22 B4, settled by source_owned_vocabularies_have_one_rust_type, just family-check (the semantic ceiling includes pse-vocabulary), governance every_crate_registered, and just codegen-check leaving no drift.
standard: core-3.1/process-simulator-1.1
scenarios: []
---

# ADR-0117: Hold the platform vocabulary enums in a crate beneath the registry and the generated model

## Context

[ADR-0115](0115-registry-typed-identities-and-vocabularies.md) Outcome 3 requires one Rust
type per vocabulary, with the generator re-exporting a vocabulary whose source is
hand-written Rust. Ten such vocabularies are hand-written in `pse-schema`:
- Namespace, Authority, SnapshotClass, DerivationGranularity, Stability, ColumnRole, InvariantKind and Severity, in `model/enums.rs`;
- Determinism, in `model/algorithm.rs`;
- OperationEffect, in `model/provider.rs`.

The generator emits a second copy of each into `pse-model`.

The dependency ceilings rule out a re-export. `pse-model` is a semantic root and must not
depend on `pse-schema`, which depends on arrow. `pse-schema` must not depend on `pse-model`,
because the generator ceiling keeps generated crates out of `pse-codegen`'s closure: a broken
generated tree would otherwise stop the generator from building. The maintainer directed on
2026-09-28 that repository rules change to fit the better design and that new crates are
acceptable.

## Scope

Adds a crate, `pse-vocabulary`, and extends the `semantic` dependency ceiling to cover it.
Its first contents are the ten platform vocabularies. It does not move `FailureClass` and
`DiagnosticCode`: `pse-model` already depends on `pse-diagnostics`, so the generator can
re-export them directly. It does not move the `pse-quantity` vocabularies, which are already
re-exported. Implementation is Plan 22 B4.

## Drivers

- **DP-01 and AP-04.** One Rust type per vocabulary, derived everywhere else.
- **DP-17.** The semantic core stays free of arrow. The generator ceiling stays, because it prevents a real bootstrap failure rather than merely restricting design.
- **AP-01.** The platform vocabularies describe the registry's own model. They belong neither to diagnostics nor to identity.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Keep two Rust types per vocabulary | Members cannot drift, because the registry reads the source's `ALL`. But conversions are needed wherever the copies meet, and ADR-0115 Outcome 3 is not met | Rejected |
| Relax the semantic ceiling so `pse-model` may depend on `pse-schema` | Pulls arrow into the semantic core | Rejected |
| Relax the generator ceiling so `pse-schema` may depend on `pse-model` | A generated tree that fails to compile would stop the generator that regenerates it | Rejected |
| Move the vocabularies into `pse-diagnostics` or `pse-ids` | Both sit beneath the two crates, but neither owns registry-model meaning | Rejected |
| **A small crate `pse-vocabulary` beneath `pse-schema` and `pse-model`** | One owner, reachable from both. The ceilings stay intact, with the new crate added to the semantic roots | **Selected** |

## Outcome

1. **The crate.** `pse-vocabulary` owns the ten platform vocabularies, each with `ALL`, `as_str`, `FromStr` and serde. It depends only on serde and thiserror/miette for its parse error, and it is a root of the `semantic` dependency ceiling.
2. **`pse-schema`** depends on `pse-vocabulary` and uses its types in the registry model. The ten local definitions are deleted.
3. **The generator** re-exports every vocabulary whose source is hand-written Rust, from `pse-vocabulary`, `pse-diagnostics` or `pse-quantity`, through a declared enum source (`EnumDecl::sourced`). This replaces today's name match against `pse-quantity`. Consequently `pse-model` depends on `pse-vocabulary`.
4. **Store columns.** An optional `postgres` feature on `pse-vocabulary` and `pse-diagnostics` carries the value mapping when a store column uses one of their vocabularies ([ADR-0114](0114-typed-operational-store.md) Outcome 25).

### Consequences

- The workspace gains a crate: registration, pins and the ceiling entry.
- `pse-schema` and generated `pse-model` code import the vocabularies from the new crate.

### Compensating controls

- `just family-check` enforces the ceilings.
- Governance `every_crate_registered`.
- The regeneration check.

### Confirmation

The tests in `verification:`, run in Plan 22 B4. Acceptance does not certify that work.

## Pros and cons

The crate is small, and it removes ten duplicate types without weakening either ceiling. The cost is one more workspace member.

## More information

- [Typed data contracts review](../design_review/reviews/design_review_typed-data-contracts_2026-09-28.md), TD06 and its [ADR-0117 addendum](../design_review/reviews/design_review_typed-data-contracts_2026-09-28.md#addendum-adr-0117).
- [ADR-0115](0115-registry-typed-identities-and-vocabularies.md) Outcome 3.
- The root `Cargo.toml` `[workspace.metadata.pse.dependency-ceilings]`.
- Plan 22 packet B4, whose progress is owned by the [store and typed-data execution packet](../plans/22-store-and-typed-data-execution.md).

## Status history

- 2026-09-28 — proposed and accepted under the maintainer's approval of the store and typed-data execution plan, after a change-tier addendum to the typed data contracts review (author review, Proposed evidence level).
