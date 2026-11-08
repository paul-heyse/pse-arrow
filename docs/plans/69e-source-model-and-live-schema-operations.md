---
title: "Plan69e: Source model and live schema operations"
status: in-progress
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

### Selected cases and lowering decision

**Interface-checked.** The optional-field fixture adds a descriptive field to
`canonical_projects` in the authored registry. The Rust relation and Surreal schema
and codec generators receive this change. Project creation and metadata readers
also consume the relation. The acceptance fixture must show that an existing row
without the optional field still opens and that an explicit description survives the
selected projection. A relation declaration alone cannot choose whether the field
is exposed by a project request or kept as internal metadata.
[Project declaration](../../crates/pse-schema/src/catalog/projects.rs) ·
[Registry codegen](../../xtask/src/codegen.rs) ·
[Project reader](../../crates/pse-operations/src/canonical_projects.rs)

**Interface-checked.** The rename fixture changes the Rust field
`PackResultSelection.field` to `selected_field`. The declaration also supplies the
recorded-result projection at tag 7. The fixture must account for the established
Proto and JSON names, preserve the original field-selection meaning, and prove
existing pack and result readers still interpret the same selection. A generic
rename cannot choose whether wire and JSON names should change or what compatibility
route should carry them.
[Recorded result declaration](../../crates/pse-model/src/recorded_result.rs) ·
[Recorded-result generator](../../xtask/src/codegen/recorded_result.rs) ·
[Result projection consumer](../../crates/pse-runtime/src/workflow/result_selection.rs)

Use the authored-edit path for both cases. The fixtures supply explicit choices about
field exposure and wire names. The composed CLI workflow binds the resulting
declaration and consumer changes, reports generated impact and runs selected checks.

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

### Generator staging checkpoint

**Interface-checked.** The aggregate build compiles against files written by the Rust,
recorded-result and Surreal bootstrap stages. The current check mode compares temporary
output and discards the candidate bytes. Preview needs a private snapshot of the selected
authored checkout, including its working edits and pinned build inputs, where all four stages
can run in order. Capture exact changed bytes and the before and after file identities under
the generator's declared output roots. Release the snapshot and its build target when the
staged delta is secured. [Ordered recipe](../../justfile) ·
[Generator comparison and publication](../../xtask/src/codegen.rs) ·
[Generated tree and declared roots](../../crates/pse-codegen/src/codegen/mod.rs)

**Implemented.** The native source-change receiver exposes preview, status, apply, recover and
discard. The generator retains the original authored identities, completed stages, exact
generated bytes and surviving states under one operation ID. It allows one retained build
snapshot per checkout. Successful staging releases that snapshot, and explicit discard releases
settled or failed preview custody. Apply checks the source and native-provider premises, then
publishes each owned file atomically. Recovery compares before and after identities and refuses
a third state. The portable rename fixture retains the established JSON and Protobuf names.
Complete the real authored-case journeys before retiring the manual route.
[Source workflow](../../xtask/src/codegen/source_change.rs) ·
[Optional field fixture](../../xtask/src/codegen/source_change/fixtures/optional-project-description.patch) ·
[Rename fixture](../../xtask/src/codegen/source_change/fixtures/recorded-result-field-rename.patch) ·
[Ownership map](../../xtask/src/codegen/ownership.rs) ·
[Ipopt writer](../../xtask/src/codegen/ipopt.rs) · [CoolProp writer](../../xtask/src/codegen/coolprop.rs)

**Implemented.** The guarded semantic rename and SSR proposals can supply a reviewed
checkout and exact file digest handoff to `source-change preview --proposal PATH|-`.
Preview validates the authored postimages before preparing a generation. `ra_ap_ide`
0.0.358 returns the structured `SourceChange` used to derive those digests. Tree-sitter
and ast-grep can provide cheaper structural
anchors for declarative patterns. The generator still owns schema, wire-name and
compatibility decisions, and the receiving checks own the resulting behavior.
For a public API change, compare the existing rustdoc-backed API capture when that
projection answers a distinct compatibility question.
[Pinned rename API](https://docs.rs/ra_ap_ide/0.0.358/ra_ap_ide/struct.Analysis.html#method.rename) ·
[Rust syntax guide](../libraries/rust_analyzers/tree-sitter-rust.md) ·
[Public API guide](../libraries/rust_analyzers/rustdoc-types.md)

Clap owns CLI argument discovery, and the project and maintenance owners supply their
effect identities and installed state. Compiler internals, MIR and expanded Rust capture
remain focused diagnostic tools for a question that needs that evidence.

## E03 — Inspect, preview, apply and recover live schema

Use D01’s authenticated target selection. Inspect installed definitions passively. A rollout
preview binds exact server, namespace, database, predecessor, target declaration, relevant
module and rollout identity. Confirm the premise at application and preserve the original
operation identity. If the acknowledgement is uncertain, reconcile actual settlement against
that identity before further writes. Reuse `CanonicalStore::validate_definitions` to produce
an observed target witness and `CanonicalStore::activate_definitions` to recheck the actual definitions at
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
The Viewer-only installed catalog observer now normalizes the same catalog as activation,
checks the expected live issuer and reads the catalog and marker on one connection.
`pse observe prepare --installed-schema` selects the owning feature and
`pse project installed-schema` requests the passive observation. The physical server
instance and loaded module still need independent evidence. The current activation path
expects a listener and rollout record without a production producer on main. Complete
the owner that issues, settles and reconciles the exact nonce on the receiving runtime
before exposing activation. The isolated SurrealKit fixture establishes parity and drift
behavior. Plan68b owns version-specific inventory and the bounded wire, and Plan68d and
68e own their exact fixture and selected candidate evidence.
[SurrealKit pin](../../.config/surrealkit.toml) ·
[Pinned suite runner](https://github.com/surrealdb/surrealkit/blob/92ebc3cc89b99f00d21c7d2cbf5136d728367061/crates/surrealkit/src/tester/runner.rs) ·
[Canonical maintenance](../../crates/pse-operations/src/canonical_maintenance.rs) ·
[Surreal substrate](../dev/surreal-substrate.md#explicit-design-phase-reconstruction-and-recovery)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| E01 | complete | Existing source registry and generator consumers | Two source-change maps and authored-edit decisions recorded above. |
| E02 | in progress | E01 decision, A01 and A03, generator and affected-check owners | The real recorded-result rename passed generated equivalence, receiving checks, stale zero-write refusal, exact publication and custody release. The optional-description fixture regenerated the Rust and Surreal contracts and passed its exact fixture-free nextest check for historical and explicit row values. Its full preview and publication journey remains open. |
| E03 | blocked | Authenticated physical instance and original rollout producer, D01 target selection, A03, Plan68 version-specific receiving | A live Viewer fixture authenticated its issuer, refused a foreign issuer and detected installed event drift. Exact rollout preview, stale refusal, original settlement and fresh-witness activation remain. |
