---
title: "Plan70e: Preparation and verification scope"
status: planned
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [current-work, design, preparation, validation]
review_sources: [docs/design_review/reviews/design_review_test-assurance-target_2026-11-08.md]
---

# Preparation and verification scope

This companion implements the proposed direction in [R06](../design_review/reviews/design_review_test-assurance-target_2026-11-08.md#r06).
The [coordinator](70-test-assurance-architecture.md) owns finding dispositions and assembled
acceptance. **Interface-checked at `622261d8f`.** Ordinary authored consumers enter a process-local
`OnceLock` that captures the package inventory and every family before returning a projection.
`Listed::page` verifies the executable for each result, and `Listed::command` and the borrowed
fixture receiver each verify the full inventory. Cargo membership tests request resolved metadata
more than once. These are distinct sources of work and have different validity boundaries.
[Authored input caller](../../tests/fixtures/authored-scientific-inputs.rs) ·
[Workflow owner](../../crates/pse-host/src/test_workflow/mod.rs) ·
[Borrowed fixture receiver](../../crates/pse-operations/src/native_fixture.rs) ·
[Membership tests](../../tests/governance/tests/every_crate_registered.rs)

**Proposed.** Prepare the selected scientific closure and authenticate the inputs a receiving
operation actually consumes. Keep the protections associated with independent launch and custody.
Plan69c owns the delivered binary membership and image leasing during enumeration. Use its
scoped receiving evidence for those mechanisms. Plan70 owns selected verification after capture.
Existing release and exact failed-rerun evidence remain under their original owners. The proposed savings concern examined
inputs and repeated work. Timing and capacity require separate evidence.
[Plan69c capture boundary](69c-ordinary-testing-and-managed-results.md#analyzer-and-capture-boundaries)

## <a id="e01"></a>E01 — Resolve the selected scientific closure before materialization

Use the authored scientific catalog and production package/header admission to resolve the selected
family, including inheritance, manifest inputs, explicit documents, substitutions and transitive
dependencies. Discover enough candidate headers and directory membership to check identity,
ambiguity, version and missing inputs. Materialize package bodies, imported documents and scientific
data only for the selected closure. Preserve the existing distinction between parser-field
projections and semantic import admission. Provider requirements remain under the scientific and
test admission owners. A process-local preparation may share the immutable result within its
verified source context. Give subsequent selections their own closure and source identity.
[Catalog](../../tests/fixtures/scientific-inputs.toml) ·
[Capture owner](../../tests/fixtures/scientific-input-capture.rs)

Migrate the ordinary runtime and PID authored consumers and deliver the same current-source
contract to the compiler/modeling migration in [70a A04](70a-selection-and-execution.md#a04).
Retain explicit transport and preparation for cases that exercise actual captured bytes, including
manual capture and Plan14 consumers with that evidence contract. Keep one implementation of
authored closure resolution for the relevant projections. Retire the unconditional union capture
from ordinary consumers as they move.

Feed the existing affected preview with conservative case/fixture consumer associations derived
from the same catalog and its inputs. Keep the preview passive. Its result is a source reach
claim, with no authority over build eligibility, provider readiness or test execution. Use
known direct consumers to seed reverse Cargo reach. If a declaration, alias, dependency,
manifest or freshness premise cannot establish reach, preserve the existing conservative widening
with a reason. Prefer a lightweight source projection over adding scientific execution dependencies
to the host. Changes to the catalog or to the provenance of the projection must invalidate any
claim of exact reach. Consume the delivered Plan69c guarded analyzer result as a candidate in
the existing selection route. The analyzer's `ra_ap_ide` 0.0.358 `runnables` and `related_tests`
can nominate a target and exact test name for a known snapshot. E01 owns scientific fixture reach
and conservative widening. [A01](70a-selection-and-execution.md#a01) and
[A02](70a-selection-and-execution.md#a02) retain actual compiler and runner admission.
[Plan69c analyzer contract and evidence](69c-ordinary-testing-and-managed-results.md#analyzer-and-capture-boundaries) ·
[Affected preview](../../crates/pse-host/src/affected.rs)

Verify equal accepted package identities and bytes for selected current-source callers. Include a
transitive input, version conflict, missing declared input, changed selected file during capture,
and an unrelated large family whose bodies are outside the materialized closure. Verify affected
preview for a shared transitive input, a known single-family input, and incomplete association
data using only passive source and metadata reads. Keep existing validated routes for
Arrow-consuming compiler/modeling and runtime checks.

## <a id="e02"></a>E02 — Determine the authenticated group handoff

Use a bounded comparison on two groups, one with a normal integration helper. Inventory the
nextest executable, captured configuration, Cargo and binary metadata, runtime/provider inputs,
test executables and auxiliary images that each group can open. Include discovery and listing
performed before test-name filters apply. The pinned nextest `0.9.146` uses native `--binaries-metadata`
and `--cargo-metadata`. The matching `nextest-runner` `0.124.0` lists binary-level possible and
definite matches before it can apply case filters. Integration suites may consume auxiliary
executables exposed as `CARGO_BIN_EXE_*`. `nextest-metadata` `0.15.0` `BinaryListSummary` carries
the native binary metadata. These contracts determine which images and metadata must remain
available for listing and execution. PSE owns executable authentication, runtime input admission
and leases through physical drain. Use Plan69c's delivered binary membership, auxiliary closure
and immutable image store at their actual scope. A test-name exclusion supplies no binary exclusion.
[Nextest listing overview](../libraries/utilities/nextest.md#103-structured-test-inventory-versus-structured-execution-events) ·
[Pinned runner binary-list source](https://docs.rs/crate/nextest-runner/0.124.0/source/src/list/binary_list.rs) ·
[Pinned listing/helper source](https://docs.rs/crate/nextest-runner/0.124.0/source/src/list/test_list.rs) ·
[Host discovery](../../crates/pse-host/src/discovery.rs)

Compare candidate handoffs on the delivered image store. Prefer a bounded group projection if
the borrower can authenticate it independently. Compare it with borrowing the original immutable
selected artifact under its existing owner. Borrowing is
eligible only if the original owner can preserve byte identity, complete membership and lifetime
through actual drain, including cancellation. If both preserve the same guarantees, prefer the
group projection when it avoids whole-inventory decode or transfer with no new custody protocol.
Keep every possibly opened image in scope when native evidence cannot narrow inside a group.
If the group boundary itself cannot be authenticated from the available receipt, keep the current
larger verification for that path and identify the missing proof required before narrowing it.

Record the selected representation, owners, input identity and boundary checks. Decide using
identical admitted selection and exact rerun, independently refused substitution of selected
bytes or configuration, verification without reading provably unrelated groups, and retention
through a settled cancellation. The receiving boundary must preserve bounded private file
admission and run-only argument enforcement. The launch part of E03 consumes this decision. Its passive page
work can proceed from the existing per-traversal authority.

## <a id="e03"></a>E03 — Verify only the consumed scope at each boundary

Use the existing `Inventory::verified_tests` traversal for a passive page, authenticating each
reached image once and checking the opened file's change identity on reuse. This evidence is
limited to that traversal. Resolve the complete group from [E02](#e02)
when constructing a native nextest command and at the borrowed service receiver. Authenticate
the actual inputs of each boundary and keep complete-inventory verification for whole-inventory
callers. Replace the whole-inventory parse and hash at group-only call sites once the new
projection passes their receiving checks. The host still owns the source-built invocation and
the fixture still owns its child, effect and physical settlement. Keep one build/list capture
per admitted context and preserve exact rerun's original identity and current-source distinction.
[Inventory API](../../crates/pse-host/src/discovery.rs) ·
[Borrowed receiver](../../crates/pse-operations/src/native_fixture.rs)

Extend the existing traversal and captured-command controls with two groups and multiple rows
per image. Require each reached page image to authenticate once. Launch refuses changed selected
image, helper, metadata or context, including binaries nextest lists before name selection.
Unrelated group bytes must not expand the selected group's read/parse scope where E02 established
independence. Compare selected case identities and ensure ordinary launch, borrowed execution,
exact rerun and cancellation preserve their existing outcomes and owner-issued release. In
targeted host controls count authenticated images and decoded scope rather than infer a speedup.

## <a id="e04"></a>E04 — Narrow membership queries and justify focused build scope

Use `cargo_metadata` `0.23.1` `MetadataCommand::no_deps()` for membership governance on the real
locked workspace. It requests workspace package metadata with the resolved dependency graph
omitted. Use `Metadata::workspace_members` and the corresponding manifest paths to identify
members. Make one real-workspace control compare the same member set against both crate
directories and declared members. Exercise the stray-directory algorithm separately with a local
known member set. Keep dependency floors and resolved ownership checks on the full dependency
graph through the existing separate consumers. Replace
membership's uses of the full-graph helper and repeated acquisitions after those controls pass.
[Pinned no-deps API](https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.MetadataCommand.html#method.no_deps) ·
[Membership overview](../libraries/rust_analyzers/cargo_metadata.md#32-workspace-members-versus-all-packages) ·
[Membership tests](../../tests/governance/tests/every_crate_registered.rs) ·
[Full graph helper](../../tests/governance/tests/common/mod.rs)

For DTA-053, pick a named focused control whose admission includes a capability unrelated to its
claim. Use [A01's pinned unit-graph contract](70a-selection-and-execution.md#a01) to compare the
actual graph and an equivalent check at the existing smaller
owner. Move it only if the same behavior is exercised, the admitted closure is narrower and
explicit validation remains correct. If inspection finds no defensible candidate, record the
graph and the needed capability distinctions as the conclusion and keep the current control.
A filter can narrow execution while retaining the crate's build graph. Preserve genuine provider,
maintenance and validation variants and the retirement of scenario-only variants. Retain
Plan67d's compatible graph reuse at its recorded scope. [Build reuse evidence](67d-cargo-variants-and-sccache-reuse.md)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [E01](#e01) | planned | Authored catalog and production admission | Selected closure and authoritative conservative associations serve ordinary consumers and A04. Scoped exactness, missing/changed input and affected-preview controls pass. |
| [E02](#e02) | planned | Plan69c binary-membership facts and pinned nextest interfaces | Bounded comparison establishes safe group transfer/borrow or identifies the unresolved proof. Receiver verification and custody boundaries are recorded for E03. |
| [E03](#e03) | planned | Existing passive traversal. E02 decision for launch and borrowing | Page verifies reached images once. Selected launch and receiver authenticate their proved scope and preserve exact selection, tamper refusal and original custody. Replaced broad paths retire where moved. |
| [E04](#e04) | planned | Real workspace membership and admitted graph for a named focused control | Membership-only acquisition and local stray control pass alongside preserved full-graph checks. The focused control moves with equivalent narrower evidence, or its graph supports a documented retention decision. |
