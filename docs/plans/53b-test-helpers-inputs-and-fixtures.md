---
title: "Plan 53b: Test helpers, inputs and fixtures"
status: complete
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
scenario_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [current-work, design, validation, preparation]
---

# Plan 53b: Test helpers, inputs and fixtures

## Target and responsibility

**Proposed:** use narrow common mechanics and immutable scientific preparation to make real tests easier to understand and cheaper to construct. Owns [R02/R03](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md#r02), with [coordinator-owned](53-test-architecture-target.md#finding-dispositions) dispositions. No new test framework, fixture language, crate, shared mutable database or universal comparator is selected. Production types and admission are reused where appropriate; independent science remains independently specified.

The existing compiler already exposes `publish_modeling`, returning `Arc<ModelingRevision>` with rows, scope, documents, checked representation and compiler context. `publish_modeling_revision` avoids repeated checking while enforcing compiler-context equality, physical-owner `Arc` identity and receiving budget (`crates/pse-compiler/src/workspace/modeling.rs`). These are the selected mechanisms, not a proposed new cache. Keep blueprint §14 admission and §24.1 test ownership guarantees.

## Work packages

<a id="b01"></a>
### B01 — Shared construction and truthful observations

Create a narrow ordinary bound-source operation in `pse-math` using `BodyBuilder::prepare`, `SourceBindings::new` and `SourceBody::new`. Consumers supply explicit source attribution before compilation releases the body. Settings, cancellation, residual construction and scientific expectations remain caller-owned where different. Inventory actual duplicate call paths, migrate those with the same mechanics and delete the replaced copies. Do not infer a generalized fixture DSL from superficially similar setup.

For compiler reuse observations, first settle I03: existing `semantic_body` counters are preferred if they establish the same invariant. Otherwise centralize the private Salsa observer once, with positive cold/invalidation controls and explicit trace completeness. Missing/truncated telemetry is unknown, never zero work. Preserve the independently fresh recomputation result comparison. Acceptance distinguishes warm reuse, cold recomputation, actual invalidation and missing observation; a renamed internal query should not force scattered test rewrites. If the stable counter cannot observe the claimed mechanism, narrow its claim rather than fabricate precision.

<a id="b02"></a>
### B02 — Typed modeling refusals and valid controls

Current `ModelingError::Contract(declaration, message)` and `ModelingContract` expose prose for some consumed distinctions. Add the required stable typed reason at the source owner, preserving declaration identity, detailed diagnostic code, revision-bound location and display projection. Migrate affected constructors, forwarding/codec/projection consumers and selected substring assertions together. Do not introduce a central classifier that reconstructs meaning from messages or migrate the whole error taxonomy.

I04 checks compatibility against blueprint §23.2 and its existing typed-cause contract before changing the seam. Present source has room for source-owned typed reasons; no additional blocking rule is established by plan creation. If implementation discovers a genuine public compatibility promise that forbids the change, identify its exact impact and obtain a decision before that incompatible change. This condition does not block unrelated packages.

For DTA-006, resolve the known origin-sensitive quotient inputs to valid admitted identities/context, show the valid control succeeds, then alter the intended single condition and assert its typed reason and declaration. An unrelated missing-ID/context refusal must fail the test. Display wording changes should not break semantic assertions. Preserve distinct structural admission and numerical infeasibility.

<a id="b03"></a>
### B03 — Honest support ownership and removal of unused infrastructure

Inventory real consumers of `FaultStore`, `FaultPlan`, `CountingStore` and their execution/introspection helpers. Remove the unused machinery and unused dev dependencies; retain the fixture factory, `NativeFixture` and capture consumers. Do not force tests to adopt doubles merely to justify keeping them. Existing independent fault/borrower evidence remains.

Separate `canonical-tests` participation from already existing `test-support` instrumentation. Operations already has support ownership and runtime's dev configuration selects it. Preserve deterministic post-ACK suspension, exact retained readers and drain controls; do not turn all tests into a single production variant or add a new crate. `FixtureLifetime` and its borrowers remain real ownership controls, with explicit fixture-backed versus production-`None` and Python scope. Qualify both selected support-enabled controls and complementary support-absent production-shaped reader/drop journeys before deleting replaced feature coupling. D01/D02 must use the resulting intentional closures, not claim every difference is duplicate compilation.

<a id="b04"></a>
### B04 — One scientific family dependency and projection owner

For each affected fixture family, declare authored roots and explicit synthetic substitutions once. Derive the complete consumed source/data/import closure through the real loader and retain source identities. Share equal closures; keep bounded synthetic declaration projection distinct from authentic full-package transport. I05 resolves the sufficient family closures and absence/membership representation; minimal closures are an optional simplification only if established by actual consumption. Retain the synthetic adapter where its boundary is intentional.

Feed these same consumed dependency facts to conservative `affected` preview. Additions, deletions, import/data edits, directory membership and absent lookups must reach the right consumers; unknown edges broaden selection. Plan52 coverage correspondence remains an observation, never a replacement for dependency edges or permission to prune them. Migrate the actual runtime/compiler/conformance/Python consumers identified by DTA-027/035/036, not just one illustrative fixture.

Capture shared scientific bytes once as immutable runtime input rather than embedding the shared package repeatedly with `include_str!`. Keep small local literals and true embedding tests. A source-only edit changes captured input identity/preparation and affected selection without recompiling unrelated Rust test binaries solely for shared science. No uncontrolled per-test rereads or private duplicate source lists remain. Source identities preserve authored physical context and exact bytes, including deliberately synthetic provenance.

Acceptance: import and data-only edits, new/deleted member, missing lookup becoming present, authentic transport versus synthetic projection, unknown dependency fallback and a source-only runtime reload. Independent fresh admission tests stay fresh. Delete each replaced embedded list/copy alongside its migrated consumers; do not retain an alternative family authority.

<a id="b05"></a>
### B05 — Reuse admitted immutable input, keep attempts fresh

Reuse a compatible `Arc<ModelingRevision>` through `publish_modeling_revision` in a fresh workspace constructed from its compiler context. Keep mutable workspace state, `ModelingPackage`, runtime, database, numerical result and borrower per test/attempt where required. Validate receiving budget, exact physical owner and compiler context at reuse. A shared admitted revision never replaces original-space checks for a new numerical state. Test incompatible context/owner/budget refusals and compare fresh versus reused results. Admission, source edit, invalidation and publication controls continue to build fresh.

Move immutable preparation before durable fixture construction when the assertion does not require durable admission. Two concrete Python migrations are required: `test_native_capability_discovery_and_hard_cut` uses its existing `pse.Runtime(settings, ephemeral=True)` capability surface without canonical substrate; `test_canonical_constructor_preserves_interpretation_and_transport_timeout` consumes immutable admitted module evidence through the existing `surreal_server.study_module` preparation boundary without acquiring a database/context. Its synthetic endpoint and real module-admission interpretation remain tested. Do not weaken authentic admission to a made-up object.

The family closure from B04 supports wider migration, but an early B05 slice can reuse the already sufficient revision before minimal closure work. Preserve fresh database/schema/issuing-store identity, independent connections and drain tests (DTA-018). No blanket database templating/shared mutable store is selected. Prove the two local Python cases allocate neither store/context nor canonical group requirement; C02/F01 consume these new actual effect facts.

## Integration obligations

B02 owns DTA-006 scientific refusal closure with 53a input. B04 produces dependency facts for D04's narrow capture and C02 selection requirements. B03 produces support/configuration distinctions for D02. B05's immutable module preparation and effect-free cases let F01 remove only demonstrated barriers. Separate semantic dependency from shared editing: coordinate runtime/conformance tests, `Cargo.toml`, root/Python conftest and source loaders with the other owner before concurrent edits.

Acceptance is planned targeted compiler/math/runtime/Python evidence during execution, followed by 53g's relevant assembled journeys. Document structural removed admission/embedding/store work without claiming latency or memory gains. No mutable-workspace cache, compulsory trace campaign or uniform helper hierarchy belongs to this target.

## Execution checkpoint — 2026-10-24

B01–B05 are authorized on main under the owning session; Plan52 implementation is paused for
its independent review. Reuse 52c's analytic/stateful and cancellation controls, typed scientific
causes and coverage overlay. Reuse 52e's existing fixture staging/remapping and deterministic
regeneration; its SQL populations do not replace canonical runtime admission or borrowers.
The coordinator retains package/inquiry/finding state. The shared input contract precedes
consumer migration; independent bound-source/refusal/support work proceeds with separate file
owners. Targeted functional checks only; no new audit, benchmark or repository-health campaign.

## Verification and Outcome — 2026-10-24

**Implemented and Tested:** all B01–B05 producer/consumer migrations, replacement deletions and
targeted acceptance are complete at the boundaries below. The coordinator owns package and
finding closure; G01 retains the later joined Plan53 qualification. This companion remains
referenced by the active series' downstream C/D/F work and publication owner.

**Implemented:** B01's `SourceBody::prepare` owns common bound-source mechanics; callers retain
their scientific settings, bindings, cancellation and residual expectations. Ten duplicate
math/backend construction paths moved and their finish helpers were deleted. Compiler tests use
existing semantic-body admission/completion counters for that bounded claim and one complete
private observer for remaining query assertions. Missing or truncated observations refuse counts.

**Implemented:** B02 introduces source-owned `ModelingContractReason` and migrates consumed
refusals/forwarders. The existing diagnostic code, rule, record identity, human detail and actual
revision-bound location survive; an authored requirement retains its separate identity without
inventing a span. Unmigrated contracts explicitly remain `Unclassified`. DTA-006 now admits a real
quantity/context control before single-condition mutations assert the intended typed rule.

**Implemented:** B03 removes unused store doubles, execution/introspection helpers, self-tests
and dependencies while retaining real `NativeFixture`, factory and capture consumers.
`canonical-tests` selects participation; `test-support` owns fixture lifetime and deterministic
hooks. Runtime test closures retain support, while the actual native Python production graph
does not select it. Both support-enabled and support-absent lifetime/reader selections pass below.

**Implemented:** B04's single catalog declares nineteen families. Production bounded package
reads, typed P0 resolution and the authored parser derive source/data/import closure; exact
directory membership, absence and package inventory are retained and rechecked before publication.
Reference bytes are explicit invocation input through `PSE_SCIENTIFIC_INPUTS`; migrated Rust
consumers no longer embed shared packages. Authentic transport and synthetic declaration projection
remain separate. `affected` consumes these facts conservatively. Selected pure archives stage and
remap the exact capsule; native archival retains its existing unsupported-closure refusal.
Python maps only this checkout's reference root, preserving arbitrary user directories with a
similar spelling. Conformance, campaign, process-cost and selected pytest controllers prepare
before child execution; a supplied capsule is validated without recapture.

**Implemented:** B05 reuses only a compatible immutable `Arc<ModelingRevision>` and creates a
fresh mutable workspace from its compiler context. Independent fresh admission/edit controls
remain. The two local Python cases use an ephemeral capability runtime and authenticated immutable
study-module evidence, respectively; neither acquires a canonical context/database. Collection
and execution guards reject accidental durable dependencies, including transitive fixture/group
selection. Process-local revision reuse is distinct from invocation-wide captured byte sharing.

**Tested — targeted functional evidence, zero-failure target:** commands used the pinned checkout
environment and force-validation on actual Arrow-consuming closures. Counts below describe
separate selections and are not an additive coverage total.

| Command / selection | Result and conditions |
|---|---|
| `scripts/pse-env -- just unit-package pse-math 'test(bound_source_preparation) \| test(implicit)'` | 74 passed, zero failed; math binding/implicit controls. |
| `just unit-package pse-modeling <selected filterset> -p pse-compiler -p pse-admission --execution-effects native-local` through `scripts/pse-env` | 138 passed, zero failed, 487 filtered; typed reasons, scientific selection/composition/reaction/parameter/potential, compiler observations/revision reuse, capture and migrated authored consumers. Invocation `b43860d73011434b8a537d2d39eaa786`. |
| `scripts/pse-env -- just unit-package pse-testkit 'test(native_unit) \| test(implicit)' -p pse-backend-native --execution-effects native-local` | 15 passed, zero failed, 316 filtered. |
| `scripts/pse-env -- just unit-package pse-admission mixture_normalization_preserves_differences_and_requires_actual_fraction_contracts --test quantity_relations --execution-effects native-local` | One passed, zero failed, 29 filtered; admitted DTA-006 control and intended refusals. |
| `scripts/pse-env -- cargo t -p xtask --bin xtask --features physical-admission -E 'test(scientific_inputs_)'` | Six passed, zero failed, 84 filtered; real dependency/version/import resolution, source-only reload, projection/inheritance and source/data/inventory drift refusal. |
| `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_test_run scripts.tests.test_scientific_inputs scripts.tests.test_test_archives scripts.tests.test_native_tests scripts.tests.test_scientific_input_consumers scripts.tests.test_host_admission scripts.tests.test_thermodynamic_campaign scripts.tests.test_case_measure scripts.tests.test_case_collection.CollectionSelectionTests.test_build_selects_compiled_feature_only_when_requested scripts.tests.test_validation.ValidationTests.test_k4_smoke_selectors_build_and_dispatch_only_the_preparation_target` | Final integration: 162 passed, zero failed, including controller preparation, two distinct reference trees and existing lifecycle/capacity controls. Controlled interruption/missing-report diagnostics are expected assertions. No benchmark campaign executed. |
| `scripts/pse-env -- .venv/bin/python -m scripts.test_archives archive --package pse-modeling --lib --filter 'test(scientific_composition_unknown_and_complete_empty_are_distinct)' --output build/test-archives/p53b-scientific-role` | One selected test passed; exact scientific role staged. |
| `PSE_SCIENTIFIC_INPUTS=/unavailable-original-producer/scientific.json scripts/pse-env -- .venv/bin/python -m scripts.test_archives replay build/test-archives/p53b-scientific-role --output build/test-archives/p53b-scientific-role-replay` | One selected test passed using remapped captured bytes; historical execution, not current-source qualification. |
| `scripts/pse-env -- just py-sync-native-correctness` | Installed native force-validated extension built successfully; actual API stubs matched. `cargo tree -p pse-py --locked --features force-validate,native-solvers -e features,no-dev -i pse-operations` confirmed no `test-support`. |
| Native `just check-package` selections for compiler/modeling/admission/operations/testkit/runtime/xtask and conformance/benches | Passed affected all-target closures, including `native-acceptance`, `native-process` and `canonical-tests`. Final runtime recheck passed with zero warnings after removing an unused fixture wrapper. |
| Two local Python cases, `-n 0`, native correctness extension, no-allocation guards | Both passed through `test_run.run_python` with authenticated extension capture and the established invocation/result composer; invocation `42bb174b2292483e9b24497d471a0c4c`. An earlier direct pytest pass is supporting evidence only. |
| `test_modeling_kernel.py::test_conformance_runs_the_declared_reference_set`, `-n 0`, selected controller preparation and native correctness extension | One passed through `test_run.run_python`; invocation `30bc7580f6f449f5a01d6c9a6b7e4d95`. Authored synthetic pass/fail and manifest refusals remain real controls. |
| Selected Rust lifetime/cancellation/GDP/MILP/reference command below | Seven passed, zero failed, 1,898 filtered, 68.064 seconds; invocation `6913fe10b8964b7d8bc00d846a394bb5`. Support-enabled runtime/operations closures against the regenerated selected canonical service. |
| Five selected Python cases below, `-n 0`, native correctness extension, actual freshly built worker binding and `test_run.run_python` | Five passed, zero failed, 119.53 seconds; invocation `5817ed62465b4125bab51e40c73c6ae9`. Two local no-allocation controls and three support-absent production reader/drop journeys against that same service. |

The final Rust selection was:

```bash
scripts/pse-env --native=solver,klu,isolation,uno,petsc --store -- just unit-package pse-runtime 'test(fixture_lifetime_waits_for_last_store_borrower) | test(fixture_explicit_drain_retains_database_for_runner_disposition) | test(fixture_connections_preserve_deployment_database) | test(canonical_member_transfer_cancellation_keeps_unacknowledged_facts_out_of_graph) | test(authored_gdp_fixture_selects_the_enumerated_alternative) | test(authored_milp_routes_to_highs) | test(conditional_unit_reference_request_declares_original_inventories_and_root_refusal)' -p pse-operations --features pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-operations/canonical-tests
```

The final Python command was a native/store `scripts/pse-env` invocation of
`test_run.run_python([sys.executable, '-m', 'pytest', '-n', '0',
'--basetemp=build/p53b-production-reader-python', ...])`, with authenticated
`python_extension.capture`/`execution_environment` and the worker artifact from the preceding
native build bound through `native_tests.worker_binding`. Its exact selected node IDs were:

```text
python/pse/tests/test_native_workflow.py::test_native_capability_discovery_and_hard_cut
python/pse/tests/test_canonical_results.py::test_canonical_constructor_preserves_interpretation_and_transport_timeout
python/pse/tests/test_canonical_results.py::test_canonical_close_preserves_live_result_reader_and_independent_caller
python/pse/tests/test_canonical_results.py::test_canonical_result_selection_and_progress_reopen
python/pse/tests/test_canonical_results.py::test_canonical_close_and_idempotent_cache_clear_with_retained_payload
```

**Tested — outcome reconciliation:** `just result INVOCATION --kind product-test --json` for
both final durable invocations reports complete terminals, child exit zero, exactly seven/five
selected outcomes, zero selected-unreported subjects and no report errors. Both use established
invocation/result/resource owners. Fresh per-test databases and borrowers remain independently
owned; fixture isolation does not create a second outcome ledger.

The selected service's older server receipt initially refused admission before tests executed
(exit 125), retained as a setup failure. After explicit operator authorization to discard legacy
server contents, its authenticated supervisor quiesced/stopped the listener and drained contexts.
Current `fresh-engine --drained` retired its fixed generated database/payload/context roots in
the same selected state, reusing the qualified current main server artifact. `just canonical-init`
installed the current schema and `surreal_server.py start` restored admitted open service.
No legacy data import or weakened admission guard was used. An unnecessary preparatory server
build was cancelled; its processes and host allocation drained. Other service states were not
part of this regeneration.

Mistakes corrected: the ordinary local Rust route initially bypassed scientific preparation;
preparation now occurs after actual selection and before effects, and its selected controls pass.
Actual archive execution exposed two fixture role/environment bridges that unit controls had
missed; both were repaired before successful producer/replay execution. Pure Python module
preparation initially called live server admission unnecessarily; it now authenticates immutable
module evidence through `study_module`, preserving the separate live-server freshness guard.
Independent review also identified omitted controller routes and a path-suffix collision; these
are corrected at their actual owners rather than adding a live-source fallback.

Deliberate boundaries: fresh mutable attempts, authentic admission and borrower/drain semantics
remain. Existing valid prepared-pair helpers remain where tests independently inspect both parts;
they are not compatibility copies of the removed ordinary finish operation. Full sufficient
closures are selected without minimality or measured speed/memory claims. The user-authorized
sixteen functional allocation slots retain their existing memory ceilings. No whole-repository
health, feature powerset, parity, performance campaign or broader Plan53 qualification is claimed;
A04's independent production-consumer limitations and C–G remain with their owning plans.

**Tested — process closeout:** `scripts/pse-env -- just turn-end` passed ADR indexing and
formatting. Its unrelated historical resource cleanup attempts retained unsupported old profiles;
they were not treated as disposed or repaired as part of these functional changes.
