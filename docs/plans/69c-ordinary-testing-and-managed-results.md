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
  The ordinary workspace check matched Cargo's 128 roots and 1,245 units after recording
  the benchmark-only package as an explicit exclusion. This check requires no build or
  test enumeration. [Cargo target metadata](https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.Target.html) ·
  [Cargo unit graph](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#unit-graph)
- **Likely tests for a source change.** The prepared native snapshot already exposes
  `ra_ap_ide` 0.0.358 `runnables` and `related_tests` through `inspect entity --view tests`.
  A useful extension is to hand a selected, guarded runnable to the current
  `inspect test-selection --from-result` route, which currently resolves the containing
  package. `discover_tests_in_crate` and `discover_tests_in_file` can supply source test
  names without compiling when a consumer needs browsing. Keep conservative package reach
  for incomplete configuration, macro coverage or custom harnesses. Resolve the candidate
  to an exact target and verify its filter through nextest before narrowing execution.
  Guppy's `PackageGraph` can simplify broader package-set queries if a consumer justifies
  replacing the current traversal. [Pinned analyzer API](https://docs.rs/ra_ap_ide/0.0.358/ra_ap_ide/struct.Analysis.html#method.related_tests) ·
  [Runnable receiver](../../crates/pse-host/src/semantic/native/runnables.rs) ·
  [Dependency tooling](../libraries/rust_analyzers/rust_dependency_tooling.md)
- **Exact enumeration and working bytes.** Nextest can use `--binaries-metadata` with
  `--cargo-metadata` to enumerate an existing build. The current adapter copies every
  Cargo executable event before knowing the binary membership. One interrupted
  listing retained about 12 GB of such copies. First establish the required images
  from nextest membership and its auxiliary dependencies. The pinned runner's
  `BinaryListBuilder` classifies Cargo messages without reading executable contents.
  Its `BIN_EXE` helpers are exposed to integration tests and benchmarks. Prefer the
  existing binary-list JSON boundary. Consider the builder if classification must
  happen while the Cargo artifact producer still holds custody. Then test borrowing
  the existing prepared controls. When a test outlives the build's custody, share a
  content-addressed immutable copy by digest, with leases through actual drain. A
  metadata-only listing can borrow a build if its owner establishes stable image identity
  through enumeration. This workstation's ext4 probe refused reflinking. Compare bytes
  copied and retained, exact test and filter parity, and replacement or cancellation
  behavior on a small selection with a normal integration helper. [Nextest machine-readable listing](https://nexte.st/docs/machine-readable/list/) ·
  [Pinned runner binary-list source](https://docs.rs/crate/nextest-runner/0.124.0/source/src/list/binary_list.rs) ·
  [Pinned helper mapping](https://docs.rs/crate/nextest-runner/0.124.0/source/src/list/test_list.rs) ·
  [Current capture](../../crates/pse-host/src/discovery/capture.rs)

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
owner-managed logs. Add no generic JSON-path language. Export to stdout or a deliberate
destination for a deliverable. Update docs and receiving consumers to use result IDs and typed
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
[Eligibility predicate](../../crates/pse-host/src/paths.rs) ·
[Plan67 release](67e-resource-retirement-and-storage-verification.md#E03) ·
[Plan68 history disposition](68d-surrealdb-v4-developer-and-test-receiving.md#d04--preserve-the-history-disposition)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| C01 | complete | A01 and current Cargo, Nextest and fixture contracts | The bare default admitted the 35 ordinary workspace packages with Cargo-equivalent root and feature coverage. Focused nextest controls passed. Qualification used the graph and scoped native tests in place of full workspace enumeration. |
| C02 | complete | C01, F01, A03 and Plan67 exact custody | The original failure remained readable after its executable was released. The live owner fixture ran exactly that identity with a fresh verdict and refused incomplete, changed and missing original evidence. Separate controls preserved the exact filters and rejected target or feature replacement. |
| C03 | in progress | A01–A03, existing inspection and history | Compact lookup and selected failure, provenance and output references are available. A prepared CLI recovered both live test results by ID. Remaining receivers and deliberate export stay open. |
| C04 | complete | C03 and Plan67 release owner | The timer and foreground expiry class have live evidence. Detached and borrowed tests published their expiry clocks before explicit reconciliation. Separate drain witnesses preserved the original records. A settled interrupted foreground listing released its unpublished inventory while preserving the original result pair. |
