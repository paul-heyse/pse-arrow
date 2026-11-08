---
title: "Plan70e: Preparation and verification scope"
status: planned
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [current-work, design, preparation, validation]
review_sources:
  - docs/design_review/reviews/design_review_test-assurance-target_2026-11-08.md
  - docs/plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md
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

Publish the selected capture's source identity, input population and lifetime for
[D04](70d-scientific-references.md#d04) and [F02](70f-conformance-and-benchmark-execution.md#f02).
D04 independently verifies its consumed bytes. F02 retains separate semantic admissions
over the shared source. Both can assess their local boundaries before the capture contract
is available for integration.

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

Compare the complete compiler-unit records at applicable capture and receiving boundaries
using [A01's exact multiset comparison](70a-selection-and-execution.md#a01). A matched
compiler context leaves image, helper, metadata and custody authentication with the
receiving owner.

## <a id="e04"></a>E04 — Scope workspace governance acquisition

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

For [I03](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i03),
use the same member-scoped acquisition in one real-workspace sole-hasher control. Join on
`workspace_members` before checking the owner and direct dependencies. Report a missing owner
declaration separately from a forbidden second declaration. Exercise an absent owner, an external
namesake and a second declaration locally. Keep dependency floors, engine reach and version-family
policy on the resolved graph. [Sole-hasher control](../../tests/governance/tests/blake3_owner.rs)

For [I13](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i13),
resolve the live checkout once per consuming governance control. Pass it to inventory and relative
path rendering, deferring display work until needed. Migrate the per-path callers, including error
taxonomy and shadow-struct scanning. Require identical findings and paths in the current and
relocated workspaces, and preserve refusal for unrelated or incomplete checkouts. The caller's
runtime directory determines the checkout (DP-09, H12, H26).
[Checkout discovery](../../scripts/workspace.rs) ·
[Shared formatter](../../tests/governance/tests/common/mod.rs) ·
[Relocation controls](../../tests/governance/tests/workspace_location.rs)

## <a id="e05"></a>E05 — Reduce unnecessary test harnesses and focused build work

For [I10](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i10)
and DTA-053, start with the ungated xtask `unchanged_generation_preserves_the_source_timestamp`
case. Compare `codegen-unit-test` and an exact no-default correctness selection using
[A01's actual unit graph](70a-selection-and-execution.md#a01). Adopt the focused route only
if the same test runs with a narrower admitted graph and actual Arrow leaves retain required
validation. Check that unused prepared providers are absent. Preserve the real physical, product schema
and full ABI variants. A nextest case filter alone may leave the compilation scope unchanged.
[Recipes](../../justfile) · [Generator controls](../../xtask/src/codegen.rs)

Check the module closure and actual pinned listing of the thin `pse`, `pse-host` and
`cargo-pse-correctness` targets and the standalone `pse-deployment` candidate. For a confirmed
empty harness, Cargo's target `test = false` suppresses its default test harness while the
ordinary executable remains available to integration consumers through `CARGO_BIN_EXE_*`.
Verify the actual roots, helper images and their live launch in ordinary and exact failed
rerun. Retain `pse-engineering` and other targets that contain reachable tests. Evaluate
`pse-contract` for `doctest = false` only after actual pinned rustdoc enumeration in the
supported configurations. cargo-shear 1.14.0's test-target check can nominate candidates.
The native runner or rustdoc supplies the deciding evidence. Record the result and retain
the existing route when equivalence or a narrower scope cannot be established.
[Cargo target settings](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-test-field) ·
[Host targets](../../crates/pse-host/Cargo.toml) ·
[Runtime targets](../../crates/pse-runtime/Cargo.toml) ·
[Test-target discovery](../libraries/utilities/rust_caching.md#51-bidirectional-test-and-doctest-configuration-analysis)

#### Build and cache boundary

Keep the pinned Cargo workspace feature unification, content freshness and checkout-local
targets selected in Plan67d. Native build-script rerun conditions and actual provider and
force-validation variants still apply. Incremental compilation serves small edits. Managed
sccache 0.19.0 can restore eligible compiler outputs when they are absent. It cannot establish a
linked test, native readiness or fresh test evidence. Keep the existing policy and settings.
Hakari would duplicate the selected Cargo mechanism. A cargo-shear declaration lead needs an
actual dependency and consumer graph change before removal (H12–H14, H28).
[Build owner and release](../dev/build-performance.md) ·
[Plan67d outcome](67d-cargo-variants-and-sccache-reuse.md#outcome) ·
[Exact managed cache probe](../../scripts/sccache-source/probe.sh)

For a named recurring workload whose result could change a decision, distinguish the unchanged
warm run, small edit or absent-output recovery as appropriate. Use the existing Cargo timing
path, managed JSON cache statistics and native phase observations. Record source, toolchain,
features, provider, profile, validation, cache condition and competing work. Separate eligible
compiler requests from linking, build scripts, rustdoc, discovery, fixture setup and actual
test execution. Additional tiers, budget changes, non-incremental defaults or linker changes
remain conditional on such evidence and compatibility. [Timing route](../dev/build-performance.md#selected-timing-investigations)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [E01](#e01) | complete | Authored catalog and production admission | Selected closure and conservative associations serve ordinary consumers and A04. D04 and F02 receive source identity and lifetime. Scoped exactness, missing/changed input and affected-preview controls pass. |
| [E02](#e02) | complete | Plan69c binary-membership facts and pinned nextest interfaces | Bounded comparison establishes safe group transfer/borrow or identifies the unresolved proof. Receiver verification and custody boundaries are recorded for E03. |
| [E03](#e03) | complete | Existing passive traversal. E02 decision for launch and borrowing, A01 exact comparison at applicable receivers | Page verifies reached images once. Selected launch and receiver authenticate their proved scope and preserve exact selection, tamper refusal and original custody. Replaced broad paths retire where moved. |
| [E04](#e04) | complete | Real workspace membership and live checkout discovery | Scoped membership and sole-hasher checks, equivalent relocated diagnostics, and preserved full-graph policies. |
| [E05](#e05) | complete | A01 pinned graph and exact unit comparison, existing native runner and image custody | Named focused and empty-harness decisions supported by graph and execution evidence. Adopt only equivalent narrower routes. Retain applicable cache and validation policy. |

## Execution checkpoint

**Tested.** With the normal correctness profile, the selected xtask binary's nine
scientific-input controls passed with zero failures in
`01a1bc14-04fa-724e-b49d-86dc5e05493e`. The capsule-free compiler and modeling library
selection passed 20 with zero failures in `01a1bc18-95fa-75bb-9f9c-c3145582c751`, using
explicit force-validation through their selected development closures. The separately owned
native campaign receipt is recorded in [F02](70f-conformance-and-benchmark-execution.md#f02).

**Implemented.** For an in-process `Listed` group, the owner already holds the binary list,
same-build artifacts and selected metadata digests. Verify the two shared tool images and
every image that group can list or launch, including pre-name-filter binaries and
`CARGO_BIN_EXE_*` helpers. Supplemental fixture and group-policy listings use that same
verification boundary before invoking Nextest. The page uses one verification traversal
for its visible rows.
The borrowed service retains full-inventory verification. A narrowed cross-process handoff
would need independently authenticated original group, runner configuration and Cargo
metadata, plus continuity of the original image lifetime through borrower drain. The
discovery inventory labels dynamic runtime/provider inputs as outside its capture.

**Tested.** The selected `pse-host --lib` affected and test-workflow controls passed
13 with zero failures in `01a1bc49-858d-731b-a85a-3f0eb88bc508`. The two-group
control refused a changed helper and a pre-filter image while an absent unrelated image
had no effect on the selected group's launch. The affected controls preserve shared
transitive reach, single-family reach and conservative widening when the input identity
or association cannot be established. Exact analyzer nominations remain candidates
under the existing selection owner.

**Tested.** `scripts/pse test pse-tests-governance:every_crate_registered
pse-tests-governance:blake3_owner pse-tests-governance:workspace_location
pse-tests-governance:error_taxonomy pse-tests-governance:no_shadow_structs` passed 12 with
zero failures in `01a1bc09-ef78-7430-90e9-ddca626f0b7d`. Membership uses locked
`no_deps()` metadata. Policies that reason over resolved dependencies retain their graph.

**Interface-checked.** `scripts/pse inspect test-selection xtask:xtask --target-kind bin
--admitted --full` with the exact timestamp filter admitted 574 compiler units under
`--no-default-features` and 1,105 under `--features full-codegen`. Both graphs explicitly
validated the Arrow leaf and neither selected a native provider. This counts compiler work
admitted by the graph. It gives no elapsed-time estimate.
`just codegen-timestamp-test` ran the exact case once with zero failures in
`01a1bc22-e878-7683-bb9b-3d34aa91b5b7` under the focused correctness selection.

**Tested.** The pinned managed binary listing found zero test cases across `pse`,
`pse-host`, `cargo-pse-correctness` and `pse-deployment` in
`01a1bc4b-b0a2-7580-aecb-ab38a66827d6`. The locked correctness adapter's `pse-contract`
rustdoc listing found zero doctests. Cargo now skips those default harnesses. The selected
`pse-host:correctness`, `pse-host:boundary` and `pse-host:cli_output` integration checks
passed all three with zero failures and complete settlement in
`01a1bc52-c2aa-7681-aa7b-65413b2077da`. They still launch the captured ordinary binaries.
The selected `pse-host --lib` original-failure and setup-refusal controls passed two with
zero failures and complete settlement in `01a1bc69-ed11-7407-bca8-1565fddde025`.
The exact rerun control proves that the fixture's `test = false` binary remains available
through `CARGO_BIN_EXE_*` and absent from the harness population. The original failed case
launched it, and a fresh exact rerun after image disposal launched it again. The setup
refusal still supplies no assertion rerun grant.
