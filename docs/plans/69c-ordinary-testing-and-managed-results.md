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
| C01 | in progress | A01 and current Cargo, Nextest and fixture contracts | Bare tests select the workspace ordinary profile and the managed primary has its own profile. The complete admitted workspace journey remains. |
| C02 | in progress | C01, F01, A03 and Plan67 exact custody | Rerun verification checks an exact one-to-one identity match. The original-to-rerun journey remains. |
| C03 | in progress | A01–A03, existing inspection and history | Compact lookup and selected failure, provenance and output references are available. A prepared CLI recovered both live test results by ID. Remaining receivers and deliberate export stay open. |
| C04 | complete | C03 and Plan67 release owner | The enabled timer and foreground expiry class have live evidence. Detached and borrowed test runs both published the expiry clock before explicit reconciliation. Their separate drain witnesses preserved the original paired records, and the targeted controls exercised expiry, holds and exact-pair release. |
