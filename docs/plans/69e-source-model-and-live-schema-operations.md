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

### Exact authored and generated evidence

Apply [I03](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I03)
at the existing preview owner. Replace first-substring anchors with unique structural
locations in the exact authored bytes. Use xtask's pinned Tree-sitter 0.27.0 and Rust
grammar 0.24.2 first. Locate the `canonical_projects` tuple and its direct
`column("description", …).optional()` expression, and the direct `selected_field`
member in the `PackResultSelection` declaration inside
`pse_contract::projection_struct!`. Scope matching to those token-tree constructs.
If the grammar cannot locate both shapes without ambiguous interpretation, use xtask's
`syn` 3.0.5 for the enclosing grammar and bounded token-tree matching for the custom
body. `proc_macro2::Span::byte_range` with `span-locations` yields byte ranges outside
proc-macro execution. Account for `syn::parse_file`'s BOM and shebang handling, and
validate the offsets against the captured original. Reject missing, ambiguous or
malformed anchors before preparation. Test real grammar, a comment or string decoy,
duplicate candidates and independently known UTF-8 coordinates. Keep the existing
semantic proposal and digest guard for cross-file edits.
[Preview source](../../xtask/src/codegen/source_change.rs) ·
[Pinned Tree-sitter byte ranges](https://docs.rs/tree-sitter/0.27.0/tree_sitter/struct.Node.html#method.byte_range) ·
[Pinned Rust grammar](https://docs.rs/tree-sitter-rust/0.24.2/tree_sitter_rust/) ·
[Pinned syn parser](https://docs.rs/syn/3.0.5/src/syn/lib.rs.html) ·
[Pinned span API](https://docs.rs/proc-macro2/1.0.107/proc_macro2/struct.Span.html#method.byte_range)

Preserve a bounded projection of `inspect affected` in the preview. Carry direct
owners, graph state and widening reasons, and the selected generator route alongside
package names. Exercise an incomplete graph that widens and a direct-owner case. A
semantic reference query can explain a specific consumer while the affected owner
retains conservative selection. Use the existing configuration-bound rustdoc JSON
format 61 comparison only for a distinct public Rust compatibility question. The
generator and receiving tests own preserved JSON name, Protobuf tag, presence and
historical decode.
[Affected owner](../../crates/pse-host/src/affected.rs) ·
[Format-checked API owner](../../crates/pse-host/src/api.rs)

For changed SurrealQL postimages selected by the preview, use the existing
`surrealql-validate` parser route on the exact staged bytes while the snapshot exists.
In surrealdb-parser-next 4.0.0, `Diag` contains `kind`, primary `start` and `end`,
`label`, and optional secondary `extra`. Preserve validated ranges, bounded path
and digest in A03. Keep separate files as related findings, preserve read failures
and apply the owner redaction policy. Verify UTF-8 before an invalid token, a
secondary delimiter label and an unreadable input. `surreal db validate --stdin`
is a separate syntax comparator. Revisit SurrealQL LSP 0.6.3 only for a named
semantic question unresolved by parsing and installed checks. Account for UTF-16
ranges and its different builtins revision. Revisit embedded SurrealKit `compile`
only for a named independent normalization question. It uses an embedded memory
engine omitted by the prepared remote gRPC build, and its schema v2 digest omits
extension digest and capability attestation.
[Validator owner](../../xtask/src/surrealql_tools.rs) ·
[Pinned parser diagnostics](https://github.com/surrealdb/surrealdb/blob/284fe93afc851c3baec2c262c01289602021a70d/surrealdb/parser-next/src/parse/mod.rs) ·
[CLI syntax validation](https://surrealdb.com/docs/reference/cli/db/commands/validate) ·
[Pinned LSP contract](https://github.com/surrealdb/surrealql-language-server/blob/5aa6a12a29b7def0e7df662c09ac9203e66f5033/AGENTS.md) ·
[Pinned compile implementation](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/compile.rs)

### Optional-field receiving

Retain three distinct checks under
[I02](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I02).
Complete the guarded source apply with generated-file publication and digest checks.
Parse the selected generated SurrealQL independently of installed behavior. In an
owned disposable logical database, insert a row under the predecessor, apply the
generated field change, then verify absent description and an explicit string
round-trip. Check `NONE` and `NULL` according to the actual generated field type.
Use a named migration only for an actual stored-value transformation. The rc.1
`test --schema-from both` runner applies sync and rollout replay in separate
databases and seeds after building schema. Reuse that parity evidence for its
claim. Insert the predecessor row in an ordered fixture or explicit rollout step
to establish historical-row behavior. Keep the native fixture for product
initializer and process ownership. Consider SurrealKit `test --exec` only when
named database contract tests can reuse logical database setup with scoped tokens.
Its whitespace-split command inherits most environment variables and direct child
interruption is distinct from native descendant drain.
[Pinned suite preparation and cleanup](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/tester/runner.rs) ·
[Pinned external runner options](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/main.rs) ·
[v4 field semantics](../libraries/surrealDB/SurrealDB_v4.md#34-missing-null-and-optional-values)

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

### Passive preview and frozen inputs

[I01](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I01)
uses the existing Viewer observer from `pse observe prepare --installed-schema` and
`pse project installed-schema`. The observer authenticates the issuer and reads the
normalized catalog and marker on one connection. The preview also binds the physical
server instance, exact namespace/database, issuer and predecessor, generated target
interpretation and digest, loaded module identity, rollout identity, verified tool
generation, and the exact manifest and SQL digests. Freeze effective template variables
or disable substitution. Keep producer-owned frozen inputs immutable and append forward
steps when declarations change. Use `rollout lint` for manifest and chain checking in
private staging. SurrealKit rc.1 `sync --dry-run`, `rollout status`, and
`Rollouts::pending` call setup. `rollout up --dry-run` also acquires a lock, and
`rollout plan --dry-run` can create local directories and reports a local proposal.
Use authenticated observation for the live preview. Check that preview leaves
application admission, tool metadata and the installed catalog unchanged.
[Viewer observer](../../crates/pse-operations/src/canonical_maintenance/catalog.rs) ·
[Pin](../../.config/surrealkit.toml) ·
[Pinned sync](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/sync.rs) ·
[Pinned setup](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/setup.rs) ·
[Pinned rollout and lint](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/rollout.rs) ·
[Pinned up and lock behavior](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/rollout/up.rs)

### Owned execution and authority gate

The activation consumer requires an initializer-authorized maintenance listener and exact
pending and settled records. Current main has no production producer for those records.
The existing native lifecycle owner can drain its original holders and server, and its
stopped-state capability can reopen the same disposable fixture. Persistent configured
state needs an authenticated handoff with durable closed admission, an initializer-only
listener, and recovery from the original pending nonce. Reuse the native drain machinery
under that owner. Establish the original listener and loaded module without assuming the
predecessor already satisfies current-schema readiness. Recheck preview premises before the first
effect. The owner fences writes, drains authentic holders and the ordinary server, and
owns the isolated listener and tool child under the existing initializer and credential
contract. It issues the original pending nonce and binds the exact inputs before SQL
dispatch. The checkout's prepared SurrealKit 1.0.0-rc.1 executable uses the locked v4
remote gRPC feature closure. Evaluate that subprocess first, including the
[B02 recovery prerequisite](69b-codex-workstation-and-preparation.md#exact-v4-prerequisites).
Use the existing Plan 68d fixture as evidence that scoped authentication and supervised
gRPC tool execution compose. Before selecting live apply, verify one small authored
rollout against an owned v4 fixture. Demonstrate exact endpoint, identity and credentials,
controlled effective variables, frozen inputs, explicit `start` and `complete`,
bounded output and actual native and SQL drain. Scope credentials to the required
maintenance effects and redact their presentation. If the subprocess cannot preserve
these facts, record the failing contract, compare the pinned Rust `Rollout` API and
its SDK closure, and keep apply gated until an equivalent owner path is established.
[Maintenance contract, blueprint §20.5](../authoritative_design/sections/identity-and-publication.md#section-20-5) ·
[Existing preparation owner](../../crates/pse-host/src/surrealkit.rs) ·
[Exact v4 fixture integration](../../crates/pse-operations/tests/native_service/surrealkit.rs) ·
[Plan 68d receiving](68d-surrealdb-v4-developer-and-test-receiving.md#d02--receive-database-contracts-at-the-owned-endpoint)

### Independent settlement and activation

SurrealKit's phase journal records running state, SQL execution and completion through
separate calls. Its `repair` can promote `running_complete` without verifying each
completion step. Read the original persisted phase and step state through the owner.
Check required schema effects against the installed definitions after authentic drain.
Data steps need their domain owner's acknowledgement or independent atomic effect
witness and postconditions. Use `await_migrations` for selected named v4 rewrites,
and `INFO FOR INDEX` for declared admission-critical builds under the qualified role.
Retain unresolved operations under the original nonce with admission closed where
effects remain unknown. A tool exit, catalog equality or native drain alone supplies
insufficient effect evidence.

After complete settlement, publish the exact owned nonce-bound record expected by
`activate_definitions`. Reobserve the generated-target witness and loaded module through
`validate_definitions`, and let activation recheck the predecessor and actual installed
target before committing the marker. Reconcile an already committed activation by its
original nonce. Verify zero writes for changed predecessor, instance, target or module.
Interrupt between SQL dispatch and journal completion, and after activation effect
before receipt. Distinguish unresolved and completed responses in human and machine
formats, proving at most one activation and no blind phase resubmission. Keep compact
operation, manifest and witness references. Plan68b owns the v4 inventory and bounded
wire, Plan68c owns access-path readiness, and 68d/68e own their respective fixture and
cutover evidence.
[Pinned phase and repair behavior](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/rollout.rs) ·
[Pinned index inspection](https://github.com/surrealdb/surrealdb/blob/284fe93afc851c3baec2c262c01289602021a70d/surrealdb/core/src/exec/vm/ir_lower/info.rs) ·
[Canonical maintenance](../../crates/pse-operations/src/canonical_maintenance.rs) ·
[Surreal substrate](../dev/surreal-substrate.md#explicit-design-phase-reconstruction-and-recovery)

## E04 — Reuse coordinates in bounded structural browsing

[I06](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I06)
removes repeated whole-source line-start calculation. Compute the index once per
immutable queried source and carry it through the existing match, capture and edit
locators. Preserve original bytes and source identity, output limits, completeness,
parser recovery, cancellation and change authority. Verify exact independent UTF-8
coordinates for multiple captures and a multiline edit. The direct reuse needs
no new cache or retained artifact. Revisit native range pushdown only for a named
need. Tree-sitter 0.27.0 `set_containing_byte_range` requires all matched nodes
inside the range, while the current projection checks captured nodes. An uncaptured
parent with an in-range capture must preserve existing behavior under any change.
[Coordinate owner](../../crates/pse-host/src/source_query.rs) ·
[Structural query](../../crates/pse-host/src/source_query/native.rs) ·
[Pinned range contract](https://docs.rs/tree-sitter/0.27.0/tree_sitter/struct.QueryCursor.html#method.set_containing_byte_range)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| E01 | complete | Existing source registry and generator consumers | Two source-change maps and authored-edit decisions recorded above. |
| E02 | in progress | E01 decision, A01 and A03, generator and affected-check owners | The recorded-result rename passed its full journey. Current focused Nextest controls cover structural anchors, conservative affected evidence, staged SurrealQL validation and redacted diagnostic projection. The v4 predecessor-row probe established absent, string, `NONE` and `NULL` behavior. The guarded optional-field publication still needs a combined run through the source owner. |
| E03 | blocked | D01 authenticated selection, A03, persistent native maintenance ownership and Plan68 version-specific receiving | The focused v4 Nextest control exercised frozen rc.1 phase execution. After interruption and server restart, it independently observed the committed SQL effect and the unresolved journal step. The in-process native owner binds a passive preview to its live process and listener, catalog, module, generated target and verified tool. The configured deployment path cannot mint that physical proof or close and drain its original service. The production nonce issuer, maintenance handoff, settled activation and CLI receiving remain open. |
| E04 | complete | Existing immutable source acquisition and structural query owner | The shared source index is implemented. Focused native tests and a scoped live probe verified UTF-8 captures and multiline edit coordinates while preserving the original source bytes and bounded behavior. Native range pushdown remains conditional. |
