---
title: "Plan 53b: Test helpers, inputs and fixtures"
status: draft
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
