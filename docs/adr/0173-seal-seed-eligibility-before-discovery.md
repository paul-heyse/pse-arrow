---
id: ADR-0173
title: Seal complete original seed eligibility before discovery cutoff
status: proposed
date: 2026-10-11
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-07, DP-08, DP-18, PS-10]
blueprint: [§20.6]
review: docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a01
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A populated emitted-query plan shows eligible-attempt discovery scanning is material ordinary work.
verification: Older eligible seed behind 64 unsealed candidates; complete versus recovered/cancelled prefix; protected exact membership, retirement and lost seal acknowledgment controls.
---

# ADR-0173: Seal complete original seed eligibility before discovery cutoff

## Context

Plan 36 adopts audit A01/A02. Existing terminal attempts include incomplete cancellation and recovery prefixes; terminal status cannot supply scientific seed permission. LIMIT before eligibility can hide an older complete usable seed.

## Scope

Amend blueprint §20.6 within the existing canonical completion contract. A bounded per-attempt witness refers to the frozen acknowledged seed inventory and complete-original completion. It adds no per-seed seal loop, new scientific result, rank method or publication policy.

## Drivers

Seed discovery must find an eligible compatible candidate at its query snapshot while preserving exact per-step permission and bounded result transport. Scientific completion owns permission; discovery owns access and cannot infer permission from a terminal bit.

## Options

Retaining LIMIT before client filtering loses completeness. Setting a terminal bit on every prefix conflates lifecycle and science. Updating each seed during seal makes one terminal operation grow with inventory. Select one witness atomically bound to the frozen attempt manifest; filter it, retirement and exact compatibility before a deterministic cutoff. A separate eligible-attempt discovery relation remains conditional on actual populated access evidence.

## Outcome

The controlled seal receives explicit typed seed eligibility. Complete-original inventory eligibility is issued only after all required scientific exports and acknowledgments succeed; recovery, summaries and unavailable prefixes issue none. The witness names the same closed manifest committed by seal. The candidate selector checks witness/attempt/run/generation and retirement before LIMIT. Exact protected acquisition independently verifies the chosen header's membership and its original completion's per-step seed permission. No cached witness authorizes a later independent use.

### Consequences

Migrate every actual seal caller to an explicit eligibility choice and regenerate canonical schema/query outputs. Preserve original operation identity/deadline and exact lost-ack settlement. Current interpretation admission refuses old writers; activation may not reinterpret old terminal prefixes as eligible.

### Compensating controls

Schema activation preserves historical content and refuses ambiguity. Tests distinguish complete-original inventory from interrupted recovery, individual scientific permission from candidate discovery, and copied payload ownership from live store protection. No production deletion is authorized.

### Confirmation

Interface-checked: Plan 36 coordination evidence establishes the pre-cutoff masking counterexample and exact library query contracts. Product checks remain Proposed until the implementation records actual commands and outcomes in Plan 36b. ADR status remains proposed; user acceptance of the Plan 36 target is distinct from decision-PR status acceptance.

## Pros and cons

One bounded witness preserves atomic seal and discovery completeness, at the cost of a derived field and producer/caller migration. Exact protected membership still supplies the final scientific and retention checks.

## More information

[Plan 36b](../plans/36b-studies-coordination-and-results.md#p-b8-complete-seed-selection-and-admitted-object-transport-a01a02) owns implementation and findings. The independent audit owns A01/A02; the operational store owner remains blueprint §20.6.

## Status history

- 2026-10-11 — proposed during maintainer-authorized execution of the accepted Plan 36 target.
