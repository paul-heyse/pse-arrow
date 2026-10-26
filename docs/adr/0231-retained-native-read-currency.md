---
id: ADR-0231
title: Retain admitted native read workers with producing artifact currency
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
revisit: A supported producer input cannot be associated with its actual installed image, or a retained read cannot establish exact context and native drain.
verification: Actual editable dev/native and fixed executable producing-input controls distinguish consumed/unrelated changes and raced image replacement; real retained reads exercise changed context, stale cursors, cancellation, endpoint EOF and physical drain through both dispatch routes and the foreground CLI.
standard: {core: '3.4', heuristics: '1.0', process-simulator: '1.5'}
scenarios: [docs/plans/56-design-principle-alignment.md#s10]
---

# ADR-0231: Retain admitted native read workers with producing artifact currency

## Context

DA-015/043 and Plan56 F02/F05/F06 identify repeated host declarations and per-read source hashing, secondary import and full image digest work. The maintainer selected RC06 on 2026-10-26: a foreground endpoint/workload can retain an admitted native read worker. Current scientific permission, immutable source context and actual native drain remain required.

## Scope

Amend blueprint §21.1/§24.1 for the finite public-operation projection and ordinary local editable dev/native currency/read lifetime. Fixed product/tooling executable consumers share the producing currency contract. This does not change scientific identities, canonical v10 data, solver mathematics, physical context, native demand, workstation placement or a provider's global installation ownership. Cross-compilation, custom Maturin/PGO and unmanaged providers retain explicit conservative preparation; Plan39's withdrawn all-role proof target remains withdrawn.

## Drivers

A read page must not pay for unrelated source trees or lose an existing import/image witness, but equivalent wire schemas do not establish artifact currency. Operation metadata must state real prerequisites and effect eligibility once. A retained process must own its admitted native work and protections through completion even after its observer cancels. S10 and the existing currency/lifetime acceptance distinguish those obligations.

## Options

1. Keep a fresh child and the current broad checks for each call: simple isolation, but repeats unchanged preparation and import work.
2. Qualify actual selected producing inputs, attach them to the existing preparation receipt, and retain an eligible foreground read worker under exact context: selected because it removes repeated work while carrying existing guarantees through a real lifetime owner.
3. Add a daemon, persistent compiler cache or universal producer authority: unselected; those responsibilities and proof scope are unnecessary here.

Reuse Cargo's selected unit graph and actual compiler/build-script records with existing dependency/input readers; neither a final depfile nor artifact JSON alone proves input completeness. Preserve actual installed bytes after Maturin transformations and the worker's pre/post-import image fence. Use existing finite operation/document generation and subprocess/native settlement owners rather than another workflow language or freshness ledger.

## Outcome

An operation's responsibility-owned declaration supplies its stable identity, request/result references, receiver, defaults/bounds, effects, prerequisites and read-lifetime eligibility. Generated host projections replace their hand-maintained mirrors, retaining small typed receiver adapters.

An optimized ordinary currency receipt is publishable only after actual producing closure, relevant configuration/environment, interpreter/PyO3/Maturin and consumed native installation inputs are associated with the installed artifact. Keep current checks until that replacement is qualified. Unknown input classes cannot become a successful currency result; their conservative path stays explicit.

An eligible read worker belongs to a foreground endpoint or explicitly admitted workload. Reuse requires unchanged admitted source, physical, provider, preparation and loaded-image context. Each bounded request retains its identity, original deadline and worker/session incarnation. Changed premises retire the worker; drain and resource/protection settlement precede replacement. Old cursors cannot silently attach to a replacement. Effectful and crash-isolated operations retain dedicated execution.

### Consequences

The worker protocol needs bounded repeated requests, cancellation and shutdown, with both existing dispatch routes and the foreground CLI adopting the same contract. Reusing a worker does not retain a database transaction across pages, extend a deadline or renew scientific permission. Unrelated source edits should not invalidate the qualified producing closure; relevant membership/deletion remains an input.

### Compensating controls

Bind producer observations to the actual Cargo child, installed bytes and current loaded-image identity. Preserve source/image fences, independent request decoding, native provider pins, original protections and physical drain. Pool width follows existing admission; no new host partition, background process or ledger is introduced. Unsupported routes retain conservative behavior rather than weakening guarantees.

### Confirmation

The linked focused review assesses this design at Proposed evidence before dependent worker-retention code. F05's extractor/installed association controls precede receipt replacement; F06 actual endpoint/page/cancellation tests precede removal of per-page admission. The owning E/F/G Outcomes retain executed commands and conditions; formal ADR status is not implementation acceptance.

## Pros and cons

The selected option makes preparation and read ownership local and amortizable. It also adds an explicit session protocol and requires stronger producer/context association than a stat-only shortcut. Qualification and retirement order bound that additional responsibility.

## More information

[Plan56 coordinator](../plans/56-design-principle-alignment.md), [F02/F05/F06](../plans/56f-host-agent-and-observability-contracts.md), [G qualification](../plans/56g-qualification-and-publication.md), DA-015/043, ADR-0229 and blueprint §21.1/§24.1. Existing library/source references remain at the owning plan; no broader audit or historical campaign is restarted.

## Status history

- 2026-10-26 — Proposed after maintainer selection of RC06 and authorization to execute the remaining 56e/56f/56g scope. Formal decision status remains proposed.
