---
title: "Plan70c: Test support and contract evidence"
status: planned
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [design, validation, storage]
review_sources: [docs/design_review/reviews/design_review_test-assurance-target_2026-11-08.md]
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

**Interface-checked at `e707f70a3`.** The compiler observation records formatted Salsa events and
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

Record complete, bounded execution observations under stable identities at the compiler test
boundary. The pinned Salsa event exposes `WillExecute` and the key's `ingredient_index()`.
Use that identity only if the test can map it to the intended query through a stable supported
boundary. Otherwise keep the observation local to the owning query using a typed test-only seam.
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

## <a id="c02"></a>C02 — Resolve historical provenance and final-apply fault control

**Interface-checked at `e707f70a3`.** The v2/v3 receipt tests construct predecessor requests and
activation records using today's serializer and helpers. The existing real native lost-ACK case
commits an intermediate v2 checkpoint and reopens its original request. The current final
`apply_receipt` issues the activation write, reads back the original operation on write error,
and distinguishes a found receipt, definite absence, and readback error. The intermediate case
exercises a different transition. [Migration owner](../../crates/pse-operations/src/canonical_migration.rs) ·
[Native checkpoint control](../../crates/pse-operations/src/native_fixture/migration_tests.rs)

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
identity mutation. Use an existing pinned property harness where it fits. Preserve the distinct
roles of DSL AST generation and executing the named AST selection in the selection companion.

Extend the existing `MemTable` unpruned comparison and add the equivalent-source control at the
canonical pruning owner. Cover nullable ranges and negation, empty and multiple block/file shapes,
multiplicity, and residual filtering before limit. Compare the actual returned results against the
same data evaluated without the pruning under test. Keep deterministic invalid-statistics,
protected-schema, and omitted-decode controls. Retain a deliberately wrong provider or wrong-pruning
control to show that the comparison detects lost rows. Engine predicate semantics and canonical
statistics/physical pruning remain separate assurance boundaries. [Unpruned comparison](../../tests/engine/tests/pushdown_vs_unpruned.rs) ·
[Canonical pruning](../../crates/pse-runtime/src/workflow/result_pruning_tests.rs)

Keep the broad implementation-presence scan where it earns its scope. Check actual stable
diagnostic codes through the selected leaf and delegated owner errors, including a case that fails
when the code is absent. Reuse the existing diagnostic catalog. For compile-fail examples, name the
intended compiler error and pair it with a compiling neighboring form. Match authoritative and
operator documentation claims to the exact lifecycle and engine controls exercised. Keep the
existing architecture publication route for enduring contract wording. [Taxonomy checks](../../tests/governance/tests/error_taxonomy.rs) ·
[Diagnostics and validation](../authoritative_design/sections/operations-and-validation.md)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [C01](#c01) | planned | Existing compiler, authored input and typed error owners. Fixture receiving path only for affected fixture seams. | Checked edits and shared mechanics pass their revealing controls. Positive memo evidence, fresh semantics, and exact refusal reason are established. Superseded helper paths and public seams are removed after their consumers move. |
| [C02](#c02) | planned | Current migration and native fixture owners. Access to Git history and retained evidence. | Record the selected byte witness and final fault boundary or the exact unresolved physical witness and receiving constraint. Independent branch work can proceed. |
| [C03](#c03) | planned | C02 witness and fault decision. B02 settlement for the physical control. | Independent predecessor and three recovery branches pass. The actual final commit and missing acknowledgement are demonstrated and drained. Keep native acceptance open if the physical witness is unresolved. |
| [C04](#c04) | planned | Existing identity, pruning and diagnostic owners. Native fixture readiness for physical pruning. | Named and generated equivalence and negative controls distinguish required behavior. Stable diagnostic and compile-fail claims match the executed evidence. |

## Verification boundary

During implementation use targeted checks under the repository's correctness adapter, with
explicit Arrow validation for consuming closures. Run local compiler and support assertions,
migration unit and native service cases, and the owning identity and diagnostic controls as each
package needs them. Reserve combined engine, canonical service, governance and doctest journeys
for the coordinator's assembled qualification. Report exact admitted cases, original outcomes
and physical settlement separately. No additional product qualification or measured benefit is
established by this document.
