---
title: "Plan69e: Source model and live schema operations"
status: planned
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69e-source-model-and-live-schema-operations.md
doc_topics: [current-work, design, deployment, validation]
---

# Source model and live schema operations

This companion owns [O06](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O06).
Registry declarations own authored meaning. Generators own derived views. Canonical maintenance
owns the transition between installed definitions. Source publication and database activation
are separate committed effects with a visible handoff. Plan66 retains physical and scientific
meaning, and Plan68 owns version-specific migration selection. The
[coordinator](69-intent-first-cli-and-codex-workflows.md) owns assembled acceptance.
[Registry, blueprint §4.2](../authoritative_design/sections/schema-and-relations.md#section-4-2) ·
[Plan68 schema and authority](68b-surrealdb-v4-schema-transport-and-authority.md)

## E01 — Select two supported source-change cases

Trace an optional field addition and a field rename or interpretation change through their
authored declarations, generated Rust and recorded-result contracts, canonical Surreal schema,
and affected readers and tests. Use a representative current declaration for each case.
Record presence, compatibility, units, role, validity and applicable PS-13 checks where the
field carries physical meaning. Record whether all changes can be inferred from a typed intent
without choosing scientific meaning or silently changing compatibility.

Compare composing generator and affected checks after a human edits the declaration with
lowering a typed edit to the authoritative source. Select typed lowering for a case only if the
authored change has one unambiguous owner, the full impact can be previewed, and independently
expected outputs and refusals can be checked. Otherwise use the authored-declaration path with
one composed preview and validation action. Keep both cases in scope under that decision rule.
Novel algorithms and physical choices remain author work. A general source patch engine would
require a separate demonstrated workload.

## E02 — Preview, publish and recover source changes

Give each supported form a preview containing exact checkout revision and relevant file
identities, the authored change, generated changes, affected consumers, compatibility and
selected checks. Bind apply to that preview. Recheck all premises before the first tracked
write and reject stale inputs. Keep existing generator ordering. `just codegen` stages Rust
contracts, recorded-result contracts and canonical Surreal definitions before the full generator.
Use `just codegen-check` for equivalence and existing affected-selection and validation owners
for receiving checks. Publish generated outputs through the generator.

If an effect stops partway, report completed stages, surviving identities and exact recovery
from the first unsettled stage. Recovery verifies current source and produced artifacts before
resuming. It preserves unrelated edits and refuses conflicting changes. Verify a successful
cross-generated preview and apply, a stale preview with zero writes, and recovery after a
generation failure. Move supported manual multi-stage sequences to the composed route once
checks pass. [Generator recipes](../../justfile) · [Process-simulator design profile](../design_review/design_principles/profiles/process-simulator/principles.md)

Use A01's local typed input and A03's diagnostic projection. With miette 7.6.0,
primary and secondary labels refer to byte ranges in a single source. Use separate
`related` findings with their own source identity for multiple files. Verify a
Unicode byte span and independently expected file coordinates in the receiving
preview. Preserve bounded content and the exact partial-publication recovery unit.
[Diagnostic contract](69a-cli-contracts-configuration-and-presentation.md#a03--original-outcomes-and-safe-recovery) ·
[Miette source spans](https://docs.rs/miette/7.6.0/miette/struct.SourceSpan.html) ·
[Related findings](https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html#method.related)

## E03 — Inspect, preview, apply and recover live schema

Use D01’s authenticated target selection. Inspect installed definitions passively. A rollout
preview binds exact server, namespace, database, predecessor, target declaration, relevant
module and rollout identity. Confirm the premise at application and preserve the original
operation identity. If the acknowledgement is uncertain, reconcile actual settlement against
that identity before further writes. Reuse `SchemaMaintenance::validate_definitions` to produce
an observed target witness and `activate_definitions` to recheck the actual definitions at
activation. Require a settled rollout and a generated-target witness. A staged source preview,
phase-only witness, or drained process alone cannot establish activation eligibility.

When dispatch crosses a process boundary, carry A02's validated selection and
identity or revalidate the preview premise at the owner. The receiving check
preserves the A03 operation identity, failure code, settlement and recovery unit.
Assert that stale refusal and uncertain application remain distinguishable from
completed activation in human output and the machine envelope. Retry reconciliation
with the original operation ID and owner facts.

Use the current owned SurrealKit route to evaluate preview and replay. The pinned 1.0.0-beta.6
suite runner can prepare isolated logical databases and compare sync with rollout replay under
`--schema-from both`. Use that to establish definition parity on a small fixture. Use the
canonical maintenance owner to establish original effect, installed-state recheck, interruption
recovery and protected activation. Compare any proposed wrapper against those guarantees and
prefer composing the existing owners when they suffice. Test stale predecessor, changed target
or module, interrupted apply and exact recovery. Receive version-specific schema and transport
from Plan68 without taking ownership of its migration or qualification.
[SurrealKit pin](../../.config/surrealkit.toml) ·
[Pinned suite runner](https://github.com/surrealdb/surrealkit/blob/92ebc3cc89b99f00d21c7d2cbf5136d728367061/crates/surrealkit/src/tester/runner.rs) ·
[Canonical maintenance](../../crates/pse-operations/src/canonical_maintenance.rs) ·
[Surreal substrate](../dev/surreal-substrate.md#explicit-design-phase-reconstruction-and-recovery)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| E01 | planned | Existing source registry and generator consumers | Two source-change maps and a decision for each case, with impact, compatibility and scientific constraints recorded. |
| E02 | planned | E01 decision, A01 and A03, generator and affected-check owners | Exact preview, generated publication and recoverable partial effects for the selected forms. |
| E03 | planned | D01 target selection, A03 and canonical maintenance, Plan68 for version-specific receiving | Exact schema preview, stale refusal, original settlement and fresh-witness activation. |
