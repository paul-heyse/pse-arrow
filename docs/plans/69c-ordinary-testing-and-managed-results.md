---
title: "Plan69c: Ordinary testing and managed results"
status: in-progress
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69c-ordinary-testing-and-managed-results.md
doc_topics: [current-work, storage, validation]
---

# Ordinary testing and managed results

This companion owns [O03](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O03)
and [O04](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O04). Build
on Plan67’s original invocation custody, fresh failed rerun, source-built fixtures and
authenticated release. Plan66 owns specialized scientific scope and Plan68 owns its v4
receiving controls. The [coordinator](69-intent-first-cli-and-codex-workflows.md) owns assembled
verification. [Custody](67b-invocation-custody-and-developer-history.md) ·
[Test consolidation](67c-surrealkit-and-rust-test-consolidation.md) ·
[v4 receiving](68d-surrealdb-v4-developer-and-test-receiving.md)

## C01 — Establish the complete ordinary lane

Map workspace members and Cargo target eligibility, feature and host/target closures, Nextest
profiles and filters, and provider or fixture prerequisites. The review’s equal 36-root intent
selections do not establish the admitted population. The existing Nextest default filter
excludes managed-primary cases. Trace those and other explicit filters to owning specialist
lanes. Reuse current Cargo and Nextest declarations wherever they represent the intended split.
Change their shared declarations when a demonstrated gap would otherwise exclude an ordinary
test. Resolve the inventory and record source owners for any exclusions before changing defaults.

Make bare `pse test` select the entire ordinary workspace population, including ordinary
integration targets. Preserve explicit narrow selectors such as `--changed`, package, target
and filter. Expose named scientific, release and qualification routes for specialist work.
Provide a passive selection view that explains intended roots, capability, profile, and exclusions
without starting tests or preparing binaries. Keep admitted population as a separate owner fact.
Any missing in-lane prerequisite yields a typed nonpass. Preserve explicit force-validation of
actual Arrow closures and the required managed native and fixture admission. A pure closure
remains pure. Success requires complete selected coverage and the original test verdict.
[Selection](../../crates/pse-host/src/cli/selection.rs) · [Nextest profiles](../../.config/nextest.toml) ·
[Blueprint §24.1](../authoritative_design/sections/operations-and-validation.md#section-24-1)

Use A01's typed syntax and candidate association for the passive catalog. If the
existing package provider covers only a subset of accepted selector forms, label
that coverage and its freshness. A candidate is advisory. Keep grammar discovery,
effective test selection, admitted population, and verdict as separate facts.
[Catalog contract](69a-cli-contracts-configuration-and-presentation.md#a01--one-parser-tree-for-intent-and-discovery)

### Analyzer and capture boundaries

- **Workspace and feature admission.** Use `cargo_metadata` 0.23.1 for declared
  target roles, `test`, `doctest` and required features. Use the pinned Cargo unit graph
  for the selected operation's exact roots and feature-bearing host and target units.
  On current v4 main, `pse inspect test-selection --admitted --full` admitted 35 packages,
  128 roots and 1,273 units. The pinned `cargo test --unit-graph -Z unstable-options`
  produced identical roots, unit features and dependency edges for the workspace and
  the explicit ordinary package set. The benchmark-only package was an explicit
  exclusion. Both checks use compile-free planning. [Cargo target metadata](https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.Target.html) ·
  [Cargo unit graph](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#unit-graph)
- **Likely tests for a source change.** The prepared native snapshot already exposes
  `ra_ap_ide` 0.0.358 `runnables` and `related_tests` through `inspect entity --view tests`.
  A guarded runnable with one eligible snapshot target now nominates that Cargo target
  and an exact nextest name through `inspect test-selection --from-result`. The current
  Cargo selection owner resolves the nominated target in its containing package.
  Incomplete or ambiguous candidates keep the conservative package selection.
  `discover_tests_in_crate` and `discover_tests_in_file` can supply source test names
  without compiling when a consumer needs browsing.
  Guppy's `PackageGraph` can simplify broader package-set queries if a consumer justifies
  replacing the current traversal. [Pinned analyzer API](https://docs.rs/ra_ap_ide/0.0.358/ra_ap_ide/struct.Analysis.html#method.related_tests) ·
  [Runnable receiver](../../crates/pse-host/src/semantic/native/runnables.rs) ·
  [Dependency tooling](../libraries/rust_analyzers/rust_dependency_tooling.md)
- **Exact enumeration and working bytes.** Nextest can use `--binaries-metadata` with
  `--cargo-metadata` to enumerate an existing build. The old adapter retained about
  12 GB of executable copies in one interrupted listing. The adapter now observes
  Cargo file identities before forwarding artifact events, then acquires the images
  required by nextest's binary-list JSON. It retains `BIN_EXE` helpers for integration
  tests and benchmarks, preserves dynamic-library and build-script metadata, and
  refuses a changed required Cargo path. The enclosing prepared generation supplies
  the correctness adapter under its existing lease. Mutable Cargo and nextest paths
  feed a checkout-local immutable image store. Per-invocation links share bytes among
  active borrowers and release after verified drain and final readback. The last release
  collects the unleased image. [Nextest machine-readable listing](https://nexte.st/docs/machine-readable/list/) ·
  [Pinned runner binary-list source](https://docs.rs/crate/nextest-runner/0.124.0/source/src/list/binary_list.rs) ·
  [Pinned helper mapping](https://docs.rs/crate/nextest-runner/0.124.0/source/src/list/test_list.rs) ·
  [Current capture](../../crates/pse-host/src/discovery/capture.rs)

**Tested.** The current managed nextest run on `pse-host:cli_output` exercised the
exact result export, missing fixture and passive settings cases with 3 passes and
zero failures. Its admitted inventory contained one harness and three runtime helper
images and borrowed the prepared correctness adapter. The authenticated release left
zero image objects after drain. Focused host controls also covered a changed Cargo
image, nonmember exclusion, and simultaneous immutable leases.
`pse inspect test-selection --from-result … --admitted --lib -E …` returned an exact
parser-checked next action on the current library target and refused a changed target
source path before a build.

## C02 — Test outcome and exact failed rerun

Return a separate verdict and settlement state and the original invocation reference. Use
`pse test --rerun-failed` against the retained original report and re-resolve its exact failed
identities in the current admitted inventory. Keep an absent or ambiguous identity a typed
refusal. Preserve the current policy for features, mode and test-owner context. The rerun has
its own identity and outcome. Exercise an ordinary integration member, an explicit specialist
exclusion, a missing ordinary prerequisite, a known failure and an absent current identity.
Use current fixture lifecycle evidence where it already establishes the required boundary.
Check that any applicable original forwarded filters cross F01's argument boundary
with their token meaning intact and do not replace the exact failed identities.
Check the receiving A03 record for original identity, refusal or verdict, and settlement.
[Argument boundary](69f-parser-startup-and-argument-forwarding.md) ·
[Diagnostic projection](69a-cli-contracts-configuration-and-presentation.md#a03--original-outcomes-and-safe-recovery)

## C03 — Results addressed by identity

Use `pse history` and `pse inspect invocation` as the passive routes from exact reference or
session-scoped `latest` to a compact result. Preserve existing owner identity and redaction.
Expose typed selected details for failure cases, effective configuration origin, bounded log
excerpts and linked product outputs where the owner knows them. Use `pse inspect log` for
owner-managed logs. Export the same bounded inspection to stdout or use
`pse tools export-result ID --output FILE` for a fresh deliberate destination. Update
docs and receiving consumers to use result IDs and typed
views in place of discovering paths and parsing raw output. Verify exact, latest, unavailable,
known retired and unknown references, along with read-only behavior.

At the receiving boundary, assert that selected details preserve A03's diagnostic
relationships and redacted A02 provenance where originally captured. Report unavailable
detail explicitly. Use the owner reference and bounded projection rather than copying
raw logs, source content, or a settings dump into retained results. Exercise a held,
retired and unknown reference with this view using C04's owner policy.

## C04 — Retention and authenticated release

| Class | Contents and lifecycle |
|---|---|
| Working scratch and executable captures | Keep only the original invocation’s or current borrower’s working material. Release after actual settlement and final readback commitment. Unresolved custody remains available to its owner. |
| Ordinary compact result | Keep exact ID, reproducible selection and source context, terminal summary, required inventory facts and meaningful owner links. Propose expiry seven days after settlement for this class. |
| Failure and diagnostic detail | Keep exact failed identities and a bounded redacted excerpt while the compact result lives. A separately requested capture names its consumer, byte budget and expiry or release condition. |
| Protected product results | Keep the authoritative reference. Product leases, protections, retention and recovery govern the underlying data. |
| Retirement witness | Keep only identity and retirement disposition for a proposed seven further days after release. Offer fresh selection where available. Once the witness is gone, an unresolved lookup is unknown. |

Represent holds with the named consumer and release condition. Build on
`RetentionClass::eligible` for disposable presentation eligibility. Its boolean input cannot
prove identity, owner state or settlement. The performing owner rechecks those facts and holds
under its synchronization at release, retaining material for active borrowers and uncertain
effects. Verify clock start, expiry boundary, hold and borrower races, interrupted retry,
redaction and protected references using controlled time and isolated data.

The maintainer selected automatic expiry for this bounded, regenerable class. Keep
preview and exact explicit owner release. Use `pse tools retire-result ID` to preview,
then pass the returned `--release TOKEN` to release that exact selection. Use
`pse tools hold-result ID --consumer NAME --condition CONDITION` to set a hold and omit
the condition to clear that named hold. Use `pse tools sweep-results` for a bounded
checkout-local pass. The repository-scoped user timer runs the same owner sweep
with `--registered`. Prepare it with `pse tools prepare-result-expiry` and use
`--enable` after the isolated release and timer controls pass. Published generations and protected original evidence
retain their separate release routes. Do not migrate historical ledgers or recreate old dumps.
Defer a SurrealDB index, expiry jobs over persistent history, and a maintained summary unless a
named recurring query cannot be served adequately from compact owner records. A new decision
would establish the query, real-role selection, durability, exact retirement and physical reuse.
Changefeed retention governs mutation replay and durable-session TTL governs sessions. Any
future compact database projection must retain the issuing owner's settlement, holds and
disposal authority and establish its own retention and failure boundary.
[Eligibility predicate](../../crates/pse-host/src/paths.rs) ·
[Plan67 release](67e-resource-retirement-and-storage-verification.md#E03) ·
[Plan68 history disposition](68d-surrealdb-v4-developer-and-test-receiving.md#d04--preserve-the-history-disposition)

## C05 — Explain test selection and doctest ownership

[I04](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I04)
adds an explanation to the existing C01/A01 receiving path. Report affected packages and
their widening basis, guarded source candidates, Cargo-admitted targets, Nextest-enumerated
identities, and original verdict as distinct facts. Use the existing Cargo metadata 0.23.1
`Target` fields for target roles, `test`, `doctest`, and required features. The pinned
Cargo unit graph establishes exact selected host and target compilation units. Preserve
the existing Cargo-equivalent root predicate and advisory `ra_ap_ide` 0.0.358
`runnables` and `related_tests` handoff, including current-source and target rechecks.
Conservative widening remains the response to ambiguous or incomplete resolution.
[Cargo target API](https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.Target.html) ·
[Cargo unit graph](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#unit-graph) ·
[Pinned analyzer API](https://docs.rs/ra_ap_ide/0.0.358/ra_ap_ide/struct.Analysis.html#method.runnables)

Route doc-only candidates to the existing `just doctest` lane. Cargo eligibility,
rust-analyzer runnable identity, and rustdoc Format 2 example preprocessing are source
or planning evidence. Nextest owns binary inventory, filter execution and verdict for
its lane. Reuse its current demand-driven image selection and drain release. Check a
doctest-only target, an optional edge absent from one resolved graph, and an integration
helper outside the source match. On a scoped admission probe, require unavailable exact
inventory with its reason until explicit build/list. Preserve the review's scoped
compile-free `pse-host --lib` observation as evidence of admission only.
[Doctest and Nextest lanes](../dev/validation-assessment.md) ·
[Example preprocessing](../../crates/pse-host/src/api/doctests.rs) ·
[Nextest inventory](https://nexte.st/docs/machine-readable/list/)

Keep native Cargo unification and the existing affected traversal. Revisit Guppy
`PackageGraph` or `CargoSet` only for a named richer explanation that can replace work
and preserve optional, host, build and dev edges against the pinned Cargo graph.
General source browsing needs a named consumer. Summaries, determinator, and retained
timings need an actual comparison and a bounded retention policy. Keep permissive
`Message::parse_stream` text fallback outside the strict build-observation boundary.
[Dependency capabilities](../libraries/rust_analyzers/rust_dependency_tooling.md) ·
[Pinned Cargo message parser](https://docs.rs/cargo_metadata/0.23.1/src/cargo_metadata/messages.rs.html)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| C01 | complete | A01 and current Cargo, Nextest and fixture contracts | The bare default admitted the 35 ordinary workspace packages with Cargo-equivalent root and feature coverage. Focused nextest controls passed. Qualification used the graph and scoped native tests in place of full workspace enumeration. |
| C02 | complete | C01, F01, A03 and Plan67 exact custody | The original failure remained readable after its executable was released. The live owner fixture ran exactly that identity with a fresh verdict and refused incomplete, changed and missing original evidence. Separate controls preserved the exact filters and rejected target or feature replacement. |
| C03 | complete | A01–A03, existing inspection and history | The current managed CLI recovered exact and session-scoped original results, read bounded detail and logs, exported the selected view and refused an existing destination. Held, retired and unknown results use the existing expiry owner. |
| C04 | complete | C03 and Plan67 release owner | The timer and foreground expiry class have live evidence. Detached and borrowed tests published their expiry clocks before explicit reconciliation. Separate drain witnesses preserved the original records. A settled interrupted foreground listing released its unpublished inventory while preserving the original result pair. |
| C05 | planned | C01–C02 selection and verdict owners, A01 catalog | Each projected stage identifies its owner and availability. Focused controls preserve doctest routing, conservative reach, and exact Nextest inventory on explicit demand. |
