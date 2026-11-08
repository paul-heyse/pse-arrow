---
title: "Plan70c: Test support and contract evidence"
status: complete
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [design, validation, storage]
review_sources:
  - docs/design_review/reviews/design_review_test-assurance-target_2026-11-08.md
  - docs/plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md
---

# Test support and contract evidence

**Proposed.** Give each test explicit inputs and observations at the boundary it claims to
exercise. Use independent expectations where the production operation could reproduce the same
error on both sides. This companion develops [R03](../design_review/reviews/design_review_test-assurance-target_2026-11-08.md#r03)
and [R05](../design_review/reviews/design_review_test-assurance-target_2026-11-08.md#r05).
The [coordinator](70-test-assurance-architecture.md) owns finding dispositions and assembled
acceptance. The [fixture companion](70b-fixtures-and-process-ownership.md) owns service
construction, process settlement, and release.

## <a id="c01"></a>C01 — Make support inputs and observations explicit

**Interface-checked at `622261d8f`.** The compiler observation records formatted Salsa events and
counts query-name substrings. Its bounded trace already rejects missing or truncated evidence. The
cold control establishes activity for `Parsed`, while several no-reexecution assertions consume
other query identities. Reference package setup still rewrites dependency literals, and the math and
native backend fixtures independently construct test residual bodies. [Compiler observation](../../crates/pse-compiler/src/workspace/modeling/test_observation.rs) ·
[Reference support](../../crates/pse-runtime/tests/reference_support/mod.rs) ·
[Math fixture](../../crates/pse-math/src/implicit.rs) · [Native fixture](../../crates/pse-backend-native/src/implicit.rs)

Keep valid package construction and explicit negative inputs separate. Use the existing authored
constructors, an already-valid authored fixture, or a structured overlay where a case intentionally
changes a binding. An overlay must select the intended subject or dependency identity, check the
premise it replaces, and refuse absent or ambiguous matches. Preserve authored source identity and
lineage through the change. Exercise both the intended substitution and a different or absent
dependency that would previously leave the text unchanged. The preparation companion supplies the
selected source closure. This package owns the meaning of the inputs within that closure.

Reuse residual preparation only for genuinely shared mechanics. Keep axes, derivative order,
limits, and relevant source/backend options at the call site. Preserve independent analytic and
finite-difference expectations. Reuse the smallest existing test-support boundary that can accept
these inputs without importing a solver or service for a pure control. Delete duplicate builders
when callers share an equivalent path. Keep distinct constructors when their visible differences
carry the test's scientific claim.

Record complete, bounded execution observations at the compiler test boundary. Vendored Salsa
0.28.5 `EventKind::WillExecute` exposes the database key when a query runs, and
`DatabaseKeyIndex::ingredient_index()` supplies a typed ingredient identity. This can replace
debug-name matching if a stable supported mapping to the intended query exists. Confirm that
mapping with the pinned source and a cold control that distinguishes the intended query from an
adjacent one. If the mapping cannot be established, keep the observation local to the owning
query using a typed test-only seam. This decision gates the replacement observation in C01.
[Runtime events overview](../libraries/scientific_calcs/salsa.md#141-runtime-events) ·
[Vendored event](../../vendor/salsa/src/event.rs) · [Vendored key](../../vendor/salsa/src/key.rs)

Replace debug-string matching. For each asserted absence of new work, first establish a positive
execution of the same query or phase. Verify the warm result against fresh semantic checking and
challenge a changed input that must run the affected phase. Missing, unrecognized, and truncated
observations each fail the witness.

Assert a typed reason and, where relevant, positive evidence that the intended boundary was
reached. Apply this to the sampled modeling and host refusals. Keep rendering assertions separate.
Move pure invalid-input cases to the smallest actual owner. Remove fixture-only production
surfaces when normal ownership covers their consumers, after the fixture companion has supplied
the actual receiving path where needed. A private `cfg(test)` stage seam can remain if it enables
a uniquely discriminating branch test. It grants no evidence of physical settlement.

For [I04](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i04),
construct the lifecycle query-budget fixture's two UInt64 columns directly with Arrow 59.3.0
`UInt64Array` and `RecordBatch::try_new` under the existing relation schema. The Arrow API
assembles the physical batch. Keep the normal semantic validation and candidate admission,
which establish the relation and query guarantees. Preserve the 200,000 rows, both column
values, 64 KiB scan headroom, typed pressure, cancellation and exact release. Keep the
distinct eight-row operator-expansion and 32,768-row recovery checks. The authored-literal
and bad-encoding controls continue to exercise their actual codecs. Compare the resulting
fixture and negative outcomes under force-validation before removing the tagged JSON
construction from the budget control (AP-06, H7, H10, H26).
[Pinned batch constructor](https://docs.rs/arrow-array/59.3.0/arrow_array/struct.RecordBatch.html#method.try_new) ·
[Typed fixture example](../../benches/benches/native_cache.rs) ·
[Lifecycle fixture](../../tests/lifecycle/tests/support/mod.rs) ·
[Budget controls](../../tests/lifecycle/tests/query_memory_budget.rs)

## <a id="c02"></a>C02 — Resolve historical provenance and final-apply fault control

**Interface-checked at `622261d8f`.** The v2/v3 receipt tests construct predecessor requests and
activation records using today's serializer and helpers. The existing real native lost-ACK case
commits an intermediate v2 checkpoint and reopens its original request. The current final
`apply_receipt` issues the activation write, reads back the original operation on write error,
and distinguishes a found receipt, definite absence, and readback error. The intermediate case
exercises a different transition. [Migration owner](../../crates/pse-operations/src/canonical_migration.rs) ·
[Native checkpoint control](../../crates/pse-operations/src/native_fixture/migration_tests.rs)

Keep the selected SurrealDB 4.0.0 transaction and canonical gRPC path for the physical control.
`MigrationScript` uses `BEGIN` and `COMMIT` to publish activation within the original write.
`apply_receipt` owns the exact-operation `SELECT` readback and verifies the returned receipt
against the original request. The transaction supplies the commit boundary. PSE owns original
identity, acknowledgement classification, readback and the decision against replay. C03 uses
that contract with the caller-local suppression seam selected in the
[execution checkpoint](#execution-checkpoint).
[v4 transaction overview](../libraries/surrealDB/SurrealDB_v4.md#6-transactions-isolation-and-concurrent-state) ·
[SurrealDB transaction contract](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions) ·
[Exact-request activation and readback](../../crates/pse-operations/src/canonical_migration.rs)

Search the retained fixtures and evidence for original v2/v3 request and activation bytes.
Follow the actual predecessor producer and its pinned schema or digest through Git history.
If original bytes are unavailable, consider executing the identified predecessor producer in a
disposable isolated checkout under its real pins. Record the producer revision, case inputs,
original byte hashes, and the independently expected version, digest, fingerprint, issuer,
operation, and read-only scope. Use original bytes only when this attribution is established.
Otherwise select independently authored specification witnesses with these expectations recorded
separately from the current writer. Label them as format and contract evidence. No claim of
original producer execution follows from that fallback.

Compare the existing native fixture seam with an interruption at the actual transport receiving
boundary. Select the narrowest owner-local seam that can deterministically observe the same
request committing before its acknowledgement is suppressed. It must exercise the original
error/readback path, leave the physical service able to return a durable original receipt, and
distinguish the final activation from the intermediate checkpoint. Reuse the private service and
the fixture companion's settlement contract. Reject a timer-only interruption or any seam that
reports a simulated commit as native evidence. If the real witness cannot yet be realized, record
the exact unavailable observation and receiving constraint. C03 can develop the deterministic
branch controls, while the physical lost-ACK acceptance stays open.

C02 completes when the source and provenance choice and the fault-control decision are recorded
with the evidence above. The chosen controls need only be shown feasible within this inquiry.
C03 owns their full implementation, migration of the old assertions, and acceptance.

## <a id="c03"></a>C03 — Verify predecessor requests and final-apply recovery

Use C02's selected byte witnesses and independent expected facts. A supported historical
request or receipt retains its original version, digest, fingerprint, issuer, operation and
applicability. Challenge wrong issuer, mutated or retargeted input, and an attempt to use the
predecessor as current activation authority. Remove current-writer setup from these historical
assertions once the new witness passes. Keep separate current round trips, frozen predecessor
declarations, legacy capture cleanup, and real data-bearing migration and restart controls.
Each supports its own claim.

Exercise final apply with a write error and an authentic original receipt, definite receipt
absence, and an unavailable or failed readback. Assert the verified original receipt in the first
case, the original write error in the second, and an unresolved result carrying both errors and
the same request in the third. Challenge a foreign receipt and prove no replay after failed
readback. Owner-local branch controls can inject the outcomes. The native case must separately
commit the final activation before suppressing the acknowledgement, observe the original receipt
from the real service, and settle the private service under [B02](70b-fixtures-and-process-ownership.md#b02).
Keep the original checkpoint lost-ACK case as evidence for the earlier transition. Avoid broadening
a product fault flag or mistaking a replayed write for recovery.

## <a id="c04"></a>C04 — Challenge pruning, identity, and governance claims independently

Use bounded, replayable generated cases alongside the named deterministic controls. At their
owning encoding boundaries, exercise admitted slices and offsets, dictionary repacks, nested
nulls, and partition choices under the identity that each representation claims to preserve.
Derive expected equivalence from the declared semantic inputs and include a meaningful value or
identity mutation. Use proptest 1.11.0's `TestRunner` with a bounded case count and reproducible
seed for the generated identity controls. The runner generates and shrinks counterexamples.
PSE owns admissible data, the comparison property and the deliberate mutation. Retain enough
case information to replay failures. Preserve the distinct
roles of DSL AST generation and executing the named AST selection in the selection companion.
[Pinned runner and configuration](https://docs.rs/proptest/1.11.0/proptest/test_runner/index.html) ·
[Existing seeded controls](../../crates/pse-authoring/tests/dsl_roundtrip.rs)

Extend the DataFusion 55.1.0 `MemTable::try_new` comparison and add the equivalent-source
control at the canonical pruning owner. The reference exposes all rows and lets the same
DataFusion engine evaluate the residual predicate. `TableProvider::supports_filters_pushdown`
controls whether DataFusion can drop that residual for `Exact` or must retain it for `Inexact`
and `Unsupported`. This isolates pruning behavior while sharing SQL semantics. PSE owns
the complete reference rows and multiset comparison. Cover nullable ranges and negation, empty and multiple block/file shapes,
multiplicity, and residual filtering before limit. Compare the actual returned results against the
same data evaluated without the pruning under test. Keep deterministic invalid-statistics,
protected-schema, and omitted-decode controls. Retain a deliberately wrong provider or wrong-pruning
control to show that the comparison detects lost rows. Engine predicate semantics and canonical
statistics/physical pruning remain separate assurance boundaries.
[Pushdown overview](../libraries/analytical_infrastructure/datafusion.md#42-predicate-projection-and-limit-pushdown) ·
[Pinned provider contract](https://docs.rs/datafusion/55.1.0/datafusion/catalog/trait.TableProvider.html#method.supports_filters_pushdown) ·
[Pinned memory table](https://docs.rs/datafusion/55.1.0/datafusion/datasource/struct.MemTable.html) ·
[Unpruned comparison](../../tests/engine/tests/pushdown_vs_unpruned.rs) ·
[Canonical pruning](../../crates/pse-runtime/src/workflow/result_pruning_tests.rs)

Keep the broad implementation-presence scan where it earns its scope. Check stable codes through
selected leaf and delegated owner errors using miette 7.6.0 `Diagnostic::code()`. The trait permits
an absent code. PSE's `TypedDiagnostic` adapter and catalog supply stable meanings and define
forwarding, so include an absent-code failure control. For compile-fail examples, name the
intended compiler error and pair it with a compiling neighboring form. Match authoritative and
operator documentation claims to the exact lifecycle and engine controls exercised. Keep the
existing architecture publication route for enduring contract wording. [Taxonomy checks](../../tests/governance/tests/error_taxonomy.rs) ·
[Pinned diagnostic contract](https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html#method.code) ·
[Typed diagnostic adapter](../../crates/pse-diagnostics/src/lib.rs) ·
[Diagnostics and validation](../authoritative_design/sections/operations-and-validation.md)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [C01](#c01) | complete | Existing compiler, authored input, relation admission and typed error owners. Fixture receiving path only for affected fixture seams. | Checked edits, shared mechanics and typed lifecycle fixture pass their controls. Positive memo evidence, fresh semantics, and exact refusal reason are established. Superseded helper paths and public seams are removed after their consumers move. |
| [C02](#c02) | complete | Current migration and native fixture owners. Access to Git history and retained evidence. | Independent format witnesses and the final-apply fault boundary selected. Physical receiving evidence belongs to C03. |
| [C03](#c03) | complete | C02 witness and fault decision. B02 settlement for the physical control. | Independent predecessor and three recovery branches pass. The actual final commit and missing acknowledgement are demonstrated and drained. Keep native acceptance open if the physical witness is unresolved. |
| [C04](#c04) | complete | Existing identity, pruning and diagnostic owners. Native fixture readiness for physical pruning. | Named and generated equivalence and negative controls distinguish required behavior. Stable diagnostic and compile-fail claims match the executed evidence. |

## Execution checkpoint

**Implemented.** C01 landed generation-bound typed Salsa query observations, a checked
authored dependency overlay, shared residual preparation, and direct Arrow lifecycle input
construction in `7b9358df8`. The all-targets check of `pse-modeling`, `pse-engine`, `pse-math`
and `pse-tests-engine` passed with zero failures. The resumed all-targets check of
`pse-compiler`, `pse-tests-lifecycle`, and `pse-backend-native` with KINSOL and root isolation
passed with zero failures in `01a1bbb3-1baa-7608-a0e1-cfcdeed04a8f`. The typed memo,
dependency-overlay and math residual controls passed in the ten-case managed support run
`01a1bbb6-00f3-7175-8be3-b805e1d1f7af`, which had one modeling refusal assertion failure.
`8dad80beb` checks the typed cause at the authored declaration. Its exact rerun passed
with zero failures. The three force-validated query-memory controls passed with zero failures
in `01a1bbbb-cdb1-755a-91ce-7a3e92e0feb3`. Three native residual controls passed with zero
failures in the managed KINSOL/root-isolation backend selection `01a1bbd1-f708-7241-97c0-17a59c7f5922`.
E01's demand-selected closure remains a separate prerequisite for its consumers.

**Decision recorded.** C02 selected four independently authored v2/v3 format witnesses
based on the historical wire definitions. They establish the request and activation format
with fixed expected identities and digests. Their provenance is specification evidence.
The selected test-only seam accepts the actual final transaction's verified receipt and
suppresses that acknowledgement at the caller boundary. The production error path then
reads the original operation. The private-service test must establish the actual commit and
post-reopen readback under B02's settlement contract.
[Format witnesses](../../crates/pse-operations/src/migrations/predecessor-wire-witnesses.json) ·
[Final-apply owner](../../crates/pse-operations/src/canonical_migration.rs)

**Implemented with unit evidence.** C03 landed in `b06f21516`. The default managed,
force-validated `pse-operations --lib` selection passed the historical format and final
readback controls with zero failures. The native final-apply and intermediate checkpoint
controls both passed in the default managed `native-service` selection
`01a1bba9-4c33-7283-b8cb-476b7c909e71`. The original final receipt was read back after
reopen, and both target-owned attempts recorded physical settlement. The one failure in
that wider selection was the separate B03 borrower probe, subsequently corrected.
[Native migration controls](../../crates/pse-operations/src/native_fixture/migration_tests.rs)

**Implemented with bounded evidence.** C04 added independent engine and canonical pruning
comparisons, seeded row-token equivalence at the columnar encoding owner, stable diagnostic
checks and exact compile-fail expectations in `068494575`, `48fd834d6` and `ca1eaa011`.
`64a4b335e` aligned the architecture's claims with their owning controls. The all-targets
engine check established compilation. The managed selection `01a1bbb2-1104-72db-8794-e1807e1289b3`
passed the seeded row identity, engine pruning comparison and wrong-oracle control, engine
diagnostic identity and two governance taxonomy cases with zero failures. The correctness
adapter's locked doctest run across `pse-diagnostics`, `pse-ids`, `pse-model`, and `pse-runtime`
passed 27 examples, including the changed compile-fail cases and their compiling neighbors.
Three native canonical pruning controls passed in the combined default selection. The fourth
comparison passed its SQL assertions but initially retained its protected reader through a
shared catalog registration. `8b9fb38f7` isolates that registration in a private DataFusion
catalog while preserving the snapshot's configured pool and execution state. The exact rerun
`01a1bbe6-1f12-7015-8039-22a5969f7d3e` passed with zero failures and complete settlement.
[Native comparison](../../crates/pse-runtime/src/workflow/result_pruning_tests.rs)

## Verification boundary

During implementation use targeted checks under the repository's correctness adapter, with
explicit Arrow validation for consuming closures. Run local compiler and support assertions,
migration unit and native service cases, and the owning identity and diagnostic controls as each
package needs them. Reserve combined engine, canonical service, governance and doctest journeys
for the coordinator's assembled qualification. Report exact admitted cases, original outcomes
and physical settlement separately. No additional product qualification or measured benefit is
established by this document.
