---
title: "Plan70f: Conformance and benchmark execution"
status: planned
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [current-work, design, validation, performance]
review_sources: [docs/plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md]
---

# Conformance and benchmark execution

The [coordinator](70-test-assurance-architecture.md) owns Plan70 dispositions and qualification.
[Plan66l L01/L03](66l-required-population-and-assembled-acceptance.md#packages) owns the required
scientific population and assembled scientific acceptance. Its L04 owns complete-operation
benefit measurements when claimed. This companion changes how conformance accumulates and
prepares evidence, and how the benchmark owners select and execute their workloads. Success
on a selected case or smoke route establishes evidence for that selection.

**Interface-checked at `1835fcf3a`.** The conformance report reserves the maximum check and
failure vector capacities once for the global report and again for every active fixture. The
seed manifest permits 100,000 checks and sixteen concurrent workers. Compaction after completion
reduces ordered-buffer retention. The same seed manifest selects the physical package as both
the physical root and a modeling dependency. The driver captures and loads it at each use.
The default benchmark smoke recipe omits the feature-gated `native_process` target and reaches
modeling preparation cases that require a canonical fixture it does not supply. These are
source-level observations about repeated work and actual route selection.
[Report owner](../../crates/pse-runtime/src/workflow/modeling/conformance.rs) ·
[Seed manifest](../../packages/reference/conformance.toml) ·
[Campaign owner](../../xtask/src/development/conformance.rs) ·
[Benchmark manifest](../../benches/Cargo.toml) · [Smoke recipe](../../justfile)

## <a id="f01"></a>F01 — Grow conformance report capacity with retained evidence

[I05](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i05)

Keep each running fixture's report under its existing owner. Let the run consume completed
reports in authored order through the existing ordered buffer. Grow check rows and diagnostic
slots independently in bounded increments as they are needed, in both local and global reports.
Reserve for the intended allocation before materializing capacity, account for capacity
rounding and old/new storage that can coexist, and release surplus once its storage is gone.
Prepare the charges for a check, its owned message and any associated diagnostic before
publishing their linkage. On refusal, leave the independent fixture inventory and previously
recorded checks readable with a valid diagnostic ordinal sequence. During merge, charge
destination growth while the source buffers are live. Transfer the charge of retained payloads
with the payloads and release source-only capacity at its last use. Retire the eager vector
allocations and accounting that relies on them. Keep compaction only where it releases meaningful
charged slack before ordered retention.

`pse-columnar` re-exports DataFusion Execution 55.1.0's `MemoryReservation`. Use the existing
fallible `try_grow` pool admission, `split` to transfer part of the charge under the same pool
registration, and `shrink` or drop to release it. These operations govern the reservation ledger.
The report owner must still connect the charge to fallible allocation and actual buffer capacity,
including rollback if allocation fails after admission. Use the existing typed resource-limit
and cancellation outcomes and the admitted worker drain. Preserve local and global caps,
independently recorded fixture statuses, original run identity, ordered checks and global failure
ordinals. Apply the storage change to every report constructor, including the pure consumer.
[Pinned reservation source](https://docs.rs/crate/datafusion-execution/55.1.0/source/src/memory_pool/mod.rs) ·
[Columnar re-export](../../crates/pse-columnar/src/lib.rs) ·
[Report owner and existing controls](../../crates/pse-runtime/src/workflow/modeling/conformance.rs) ·
[Pure report consumer](../../crates/pse-runtime/src/workflow/modeling/pure.rs)

**Export decision.** Keep the current all-projections-first export as the default. The bounded
inquiry at `RunReport::export` compares it with one checked projection live at a time. A projection
refusal currently publishes nothing. A writer or destination refusal can leave an ordered prefix
of complete files. Arrow IPC 59.3.0 `FileWriter::finish` writes the footer and closing marker.
The existing tempfile 3.27.0 `NamedTempFile::persist_noclobber` publishes each finished file
without replacing an existing destination. The campaign owns the ordering and the six-file
failure contract. Compare a private staging or preflight candidate only if it can preserve
projection-versus-writer precedence, original error attribution, schemas and rows, complete
files and the committed prefix. Account for any repeated projection work and temporary storage
and cleanup. Use a late projection refusal, an earlier writer refusal and an existing destination
to distinguish the behaviors, including when both faults are possible. Adopt a candidate only
if it preserves these outcomes and meaningfully reduces live projection storage for a named
report shape. Record a retention decision if the comparison fails. A change to publication
semantics goes through the contract owner and decision route before implementation.
[Export owner](../../xtask/src/development/conformance.rs) ·
[Pinned IPC writer](https://docs.rs/crate/arrow-ipc/59.3.0/source/src/writer.rs) ·
[Pinned temporary-file API](https://docs.rs/tempfile/3.27.0/tempfile/struct.NamedTempFile.html#method.persist_noclobber)

**Proposed verification.** Extend the runtime's parallel conformance and report transfer controls.
Use a high configured check cap, many tiny fixtures and a pool that admits their actual report
demand but would refuse eager full-capacity reservations. Compare detailed and compact fixture
outputs and serial versus out-of-order merge, allowing the minted run ID to differ between runs.
Check all statuses, failure ordinals and retained attribution. Exhaust the local and global caps
independently, then inject refusal during check or diagnostic growth and during merge overlap.
Verify typed outcomes and coverage limits, physical drain on cancellation, and pool release after
the last report owner drops. Exercise the export decision with the xtask report/CLI controls.
These checks establish reservation and publication behavior. RSS and elapsed-time claims require
measurement.

## <a id="f02"></a>F02 — Reuse the exact physical input within one native run

[I06](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i06)

Consume [E01's selected-source identity](70e-preparation-and-verification-scope.md#e01) for the
native campaign run. Resolve the physical root and modeling dependency occurrence before sharing.
Bind the actual selected root, package and version, declared membership and exact original
documents under the admitted registry and parse context. Equal package names, versions or
matching path spellings alone cannot establish identity. Keep the modeling closure's authored
order. Capture the common physical source once inside the admitted run. Refuse inconsistent
or missing selected inputs at capture under E01's verification rules. After capture, both
consumers read the admitted immutable source from that run. A later run admits its own inputs.
[Production capture](../../crates/pse-admission/src/authoring_driver/document/capture.rs) ·
[Campaign owner](../../xtask/src/development/conformance.rs)

Prefer one accounted `OwnedDocumentBundle` where both consumers can use the original
immutable parsed documents in their admitted contexts. `OwnedDocumentSet::try_from_bundles`
retains the shared package owners and charges each set's inventory. Registry and native
predicate checks at the receiving boundary still apply. Prove exact source/document identity
and shared custody through both admissions before removing the second parse. If the receiving
contexts require different preparation or sharing would require copying the retained representation,
reuse the verified authored bytes and keep the distinct preparation each context requires.
The physical quantity/property admission and provider registration still precede the separate
modeling semantic admission. Preserve the separate pure domain run with its synthetic input,
standalone execution owner and repeatability evidence.
[Owned documents and validation](../../crates/pse-admission/src/authoring_driver/document/owned.rs) ·
[Physical admission](../../crates/pse-runtime/src/workflow/physical.rs) ·
[Modeling admission](../../crates/pse-runtime/src/workflow/modeling.rs) ·
[Authored run selections](../../packages/reference/conformance.toml)

**Proposed verification.** Extend the xtask native campaign control with an overlapping
physical root/dependency. Compare accepted document IDs, original source bytes, package IDs,
physical and modeling results, and capture count. Require one original read of the common
input and, if selected, prove shared parsed ownership. Exercise changed membership or bytes
during capture, missing selected documents, and distinct roots with the same package label.
Changing disk bytes after successful capture must leave both within-run admissions on the
same admitted input. A new run must observe the change or refuse it. Retain the original
native success and refusal settlement control and the pure CLI no-clobber control. Compare
repeat pure domain runs at fresh destinations for the same scientific outcomes.
[Campaign receiving controls](../../xtask/src/development/conformance_tests.rs) ·
[CLI controls](../../xtask/tests/conformance_cli.rs)

## <a id="f03"></a>F03 — Admit the actual benchmark workload and its fixture

[I14](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i14)

Keep an ordinary smoke selection of reachable local cases without provider or canonical
fixture preparation. Use an explicit prepared-owner invocation for selected canonical
and solver cases, in either smoke or measurement mode. Resolve the workload, Cargo target
and features, and actual prepared inputs before fixture or runtime effects. Model canonical
and native solver requirements separately, including cases needing both. Use the existing
native provider admission and authenticated canonical fixture owner. The benchmark borrows
its private fixture selection, opens its disposable database through the existing receiver
and drains its work before the launching owner releases the fixture. Confirm at execution
which existing prepared invocation can carry the native Criterion image and its exact
selected inputs. If an invocation is missing, extend the prepared owner at this boundary.
Preserve the original connection-only and caller-owned cleanup contracts.
[Canonical receiver](../../crates/pse-operations/src/native_fixture/logical.rs) ·
[Prepared inputs and experiment routes](../dev/agent-environment.md#selected-reproducible-experiments) ·
[Benchmark workload registry](../../.config/preparation-cases.json)

The recipe owner updates `bench-smoke` help and the stale benchmark manifest/harness
descriptions to name the selected population. At execution, update the command description
in blueprint §24.3 through its documentation owner and normal revision route. Account for
local, prepared-only and unavailable cases using the actual target, workload and mode selection.
A prepared route asked to execute
a missing or unauthenticated required input returns its prerequisite refusal. A local route
reports which population it covers. Retire the implicit all-benchmarks smoke invocation and
the claim that it exercises every benchmark. Keep the scientific assertions and actual cleanup.

Resolve the modeling preparation workload before generating its declarations. A named
scalar selection needs no authored canonical bank. A named canonical selection authors only
its required workload, keeping the same deterministic published component prefix and required
declarations. For a named experiment compare per-iteration capture against sharing only
immutable, exact source inputs across samples when capture is outside the timed boundary.
Reuse requires identical admitted scientific input, runtime/pool custody compatible with
sample isolation and no dependence on a prior sample's semantic work. Fresh runtime and
semantic admission remain where the experiment requires them. If the comparison cannot
establish equivalence, retain per-iteration capture. Preserve the cold, warm, value-edit and
structural-edit sequence, typed resource refusals and teardown. Remove eager generation of
the unselected matrix from the affected preparation route.
[Preparation harness](../../benches/benches/modeling_preparation.rs) ·
[Native process harness](../../benches/benches/native_process.rs)

Keep Criterion 0.8.2 with the pinned `csv_output` feature, existing callbacks and raw samples.
Its native test mode executes the selected registered function for smoke. Its benchmark
mode samples it. A Cargo benchmark with `--test` still selects test mode, so a harness mode
decision based solely on the presence of `--bench` can select the wrong workload population.
Keep workload selection aligned with the actual mode. `Bencher::iter_custom` uses the callback's
returned measurement value and requested iteration count. Preserve the experiment's current
timed phases and distinguish per-callback observations, including warm-up where present,
from Criterion's measured samples. Report any full-operation timing separately with its
own boundary and baseline.
[Pinned Criterion mode handling](https://docs.rs/crate/criterion/0.8.2/source/src/lib.rs) ·
[Pinned timing API](https://docs.rs/criterion/0.8.2/criterion/struct.Bencher.html#method.iter_custom)

**Proposed verification.** Exercise a reachable local smoke case with no prepared fixture,
a prepared canonical smoke and an explicitly selected solver smoke, and refusal with each
required input absent. Check the actual executed and unavailable population, selected
authored declarations, original scientific outcomes and owner-observed cleanup. Run a named
measurement under its existing sampling policy and verify actual phase boundaries, raw
sample retention and complete drain. Compare equivalent sampled science and phase sequence
before adopting shared capture. Use the run to verify measurement scope. A claimed benefit
requires a matching baseline, including preparation and disposal when the complete operation
is the subject. [Blueprint §24.3](../authoritative_design/sections/operations-and-validation.md#section-24-3) ·
[Plan66l L04](66l-required-population-and-assembled-acceptance.md#packages)

## Assembled acceptance

Use `scripts/pse test pse-runtime --lib -E 'test(conformance_parallel_)'` for the focused
report controls, selecting `pse-runtime/solver-kinsol` for the existing linked worker cases.
Use `scripts/pse test xtask --lib --features xtask/package-fixtures -E 'test(development::conformance)'`
for campaign controls and the `xtask:conformance_cli` target with the same feature for CLI
receiving. Keep explicit force validation on every Arrow-consuming correctness route.
After the functional scope is integrated, the coordinator selects the assembled native
conformance and pure domain runs from the admitted manifest, covering six checked IPC relations
and original science, selection, outcomes and drain. Use the validated `just seed-conformance`
route with fresh report destinations, or its existing parameterized `just modeling-conformance`
route for an explicit selection and destination. Qualify local smoke and the explicit prepared
canonical and solver routes as separate receiving journeys. Record their command, mode,
features, selected and unavailable population, and failures against the zero baseline.
Plan66l's exact scientific denominator and outstanding assembled scientific acceptance
remain at their source owner.

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [F01](#f01) | planned | Runtime report and pool ownership, existing export contract | Demand-accounted rows survive cap, pressure, ordering and cancellation controls. Eager capacity is retired. The export comparison records adoption with equivalent behavior or retains the existing path. |
| [F02](#f02) | planned | Campaign and document owners, E01 selected-source identity for the native run | Exact physical input is captured once. Shared document ownership or verified-byte fallback serves both separate admissions. Pure evidence and original outcomes survive. |
| [F03](#f03) | planned | Criterion and workload owners, authenticated fixture and provider inputs for prepared cases | Local and prepared populations execute through their declared routes. Unselected authoring is retired. The capture comparison records equivalent reuse or retention, with sampled phases and cleanup verified. |
