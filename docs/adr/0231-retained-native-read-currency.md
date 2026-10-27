---
id: ADR-0231
title: Retain native reads through owner-published artifact generations
status: proposed
date: 2026-10-26
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-04, AP-07, DP-01, DP-09, DP-10, DP-18, DP-19, DP-20, DP-22]
blueprint: [§21.1, §24.1]
review: docs/design_review/reviews/design_review_retained-native-read-currency_2026-10-26.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Controlled construction or publication cannot preserve a complete immutable generation, or a retained read cannot establish exact context and native drain.
verification: Publication controls exercise successful construction, failed-publication preservation and lint-independent generation attachment; real retained reads exercise exact context, original cursors, cancellation, endpoint EOF and physical drain through both dispatch routes and the foreground CLI.
standard: {core: '3.4', heuristics: '1.0', process-simulator: '1.5'}
scenarios: [docs/plans/56-design-principle-alignment.md#s10]
---

# ADR-0231: Retain native reads through owner-published artifact generations

## Context

DA-015/043 and Plan56 F02/F05/F06 identify repeated host declarations and per-read source hashing, secondary import and full image digest work. The maintainer selected RC06 on 2026-10-26: a foreground endpoint/workload can retain an admitted native read worker. Current scientific permission, immutable source context and actual native drain remain required.

## Scope

Amend blueprint §21.1/§24.1 for the finite public-operation projection and ordinary local editable dev/native currency/read lifetime. Fixed product/tooling executable consumers share the producing currency contract. This does not change scientific identities, canonical v10 data, solver mathematics, physical context, native demand, workstation placement or a provider's global installation ownership. Cross-compilation, custom Maturin/PGO and unmanaged providers retain explicit conservative preparation; Plan39's withdrawn all-role proof target remains withdrawn.

## Drivers

Construction and publication have a controlled owner. Repeating the owner's source, environment,
Cargo-record or image validation in consumers adds no assurance to that contract. An already
published artifact remains valid when source files or incidental build logs change. A requested
interface or execution mode can nevertheless be incompatible with a valid generation.

## Options

1. Infer artifact freshness from source trees and build records during reads: superseded by the
   maintainer's explicit construction-driven direction; lint and unrelated builds can mutate those
   records without changing the issued artifact.
2. Construct and atomically publish a complete immutable generation in the existing preparation
   owner, then attach consumers to it: selected. Cargo and Maturin retain their build duties.
3. Add a universal producer ledger or another validation framework: unnecessary.

## Outcome

The operation owner continues to declare identity, request/result references, receivers,
defaults/bounds, effects, prerequisites and read eligibility once for generated projections.

Explicit preparation runs the trusted build/install path, records the resulting mode/profile,
validation and compiled features, copies its image into a private generation and atomically
publishes the typed descriptor in the existing receipt. Failure before publication commits leaves
the preceding selection usable. If replacement may already have committed when interrupted,
the constructor preserves the candidate and preceding generation; it never deletes a possibly
selected image or asks readers to reconstruct validity. Partial installation never becomes
the selected generation.
Construction establishes artifact validity once; consumers do not reproduce construction checks.

A consumer attaches the published generation once and loads that exact immutable image.
Distinct requested-interface/mode compatibility is checked at attachment. No source, environment,
Cargo dep-info/stdout or binary-digest rescan is required during reads or internal handoffs.
Mutable installation aliases and subsequent publication do not revoke an already attached image.
Selecting a newer generation explicitly drains/replaces the worker; original cursors never rebind.

Eligible workers remain foreground endpoint/workload owners with original physical/provider
context, request identity, deadline and native drain. Python serving-code edits require explicit
drain and reload before edited code serves operations, including lazy imports. Concurrent live
editing is outside that controlled editing contract; source scans are not a substitute for it.

### Consequences

Generation storage stays with the existing preparation owner. Retention lasts through attached
consumers; cleanup cannot remove their image/provider resources. No daemon, host partition,
second ledger, general validation language or measured speedup follows.

### Compensating controls

The constructor owns complete publication and failure settlement. External request decoding,
scientific permission, Arrow validation, cursor/session/context identity and actual native resource
drain retain their separate duties. They do not revalidate artifact construction. If a new failure
exposure requires validation, model the specific gap and place it at its owner once.

### Confirmation

The attributed focused review records the revised judgment. Plan56 F05/F06 qualification covers
publication failures, source/lint independence, explicit replacement and retained cursor/drain
ownership. Prior source-change and installation-alias-change refusal obligations are superseded
by this maintainer direction, rather than claimed as current acceptance.

## Pros and cons

The selected design removes redundant work and makes failure recovery explicit at the constructor.
It requires controlled immutable publication and explicit serving-code reload, responsibilities
already owned by preparation and endpoint lifetimes.

## More information

[Plan56 coordinator](../plans/56-design-principle-alignment.md), [F02/F05/F06](../plans/56f-host-agent-and-observability-contracts.md), [G qualification](../plans/56g-qualification-and-publication.md), DA-015/043, ADR-0229 and blueprint §21.1/§24.1. Existing library/source references remain at the owning plan; no broader audit or historical campaign is restarted.

## Status history

- 2026-10-26 — Proposed after maintainer selection of RC06 and authorization to execute the remaining 56e/56f/56g scope. Formal decision status remains proposed.

- 2026-10-27 — Revised under explicit maintainer direction: controlled construction/publication establishes validity once; consumer freshness inference and redundant validation are superseded. Formal status remains proposed.
