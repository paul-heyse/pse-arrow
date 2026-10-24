---
id: ADR-0222
title: Keep development knowledge in the disposable tooling corpus
status: proposed
date: 2026-10-23
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-04, AP-07, DP-09, DP-23]
blueprint: [§24.1, §23.1]
review: "not-required: short derived relation-family extension under existing tooling authority; Plan52 RC01 and RC06 explicitly selected by maintainer"
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A concrete consumer needs independent writable knowledge authority or a different deployment boundary.
verification: Two-source corpus membership, interrupted staging, independent evidence pointer, reader/importer enforcement, full-text and graph queries on one cut, and rebuild preserving native sources.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/plans/52b-local-knowledge-and-research.md#B04]
---

# ADR-0222: Keep development knowledge in the disposable tooling corpus

## Context

Plan52 introduces source-linked engineering knowledge, semantic ownership paths and selected
research/evidence references. Scientific research records concern admitted physical data and
ancestry; development knowledge must not change their meaning. Existing tooling evidence owns
private service selection and disposable generations (blueprint §24.1).

## Scope

Introduce separately declared development-knowledge subjects, exact productions, typed relationships, source contributors,
excerpts and exact corpus generations in the existing tooling database. This extends the derived relation family and its
readers, not canonical scientific authority, model hashes or terminal/result custody.

## Drivers

A question selects a meaningful owner-native concept by exact identity and follows declared links;
source versions and excerpts supply attributable current/historical evidence. Topics and text are
secondary discovery aids, and semantic similarity is unselected.
A corpus contains multiple immutable source versions at once; importing source B must not replace
source A or change evidence's independent active-snapshot pointer. Reads must survive missing
indexes through source-only access and never initialize a service implicitly.

## Options

Reuse canonical scientific research or widen EvidenceSnapshot into arbitrary notes; create a new
knowledge service/store; or add a distinct derived family using existing tooling lifecycle.
The latter preserves authority and shares actual deployment/credential/generation machinery.

## Outcome

### Consequences

Registry declarations generate tooling DDL/codecs and access indexes. Typed concepts consume native
document/scope/producer/evidence meanings; exact production triples preserve declaration-specific
outputs. Native metadata inheritance and status namespaces remain distinct. Generic tags or file
chunks do not define the domain, and undeclared links remain unknown. A separately staged corpus
activates only after exact source/edge/excerpt membership is reconciled. Knowledge-only reset
leaves evidence generations intact; explicit whole-tooling disposal affects both derived families.

### Compensating controls

Reader/importer roles, service identity, bounded requests, source cuts and cursor fences remain.
Derived links do not reopen raw artifacts or grant scientific permission. Historical instructions
are evidence, never current authorization; no global memory writer is introduced.

### Confirmation

Plan52b B04 owns actual two-source, interruption, query-plan, lifecycle and source-preservation
qualification. This source assessment is Interface-checked, not implementation or measured benefit.

## Pros and cons

One local service and current typed tooling operators are reused. New declarations, full-text
lowering and corpus-specific activation need maintenance, but avoid a second authority or general
memory product. Source-only queries remain useful when persistent access is unavailable.

## More information

[Plan52](../plans/52-agent-effectiveness-and-engineering-workflows.md) owns decisions and progress;
[52b](../plans/52b-local-knowledge-and-research.md#B04) develops consumers and acceptance.

## Status history

- 2026-10-23: Proposed before implementation under the maintainer-authorized RC01/RC06 target.

- 2026-10-23: Maintainer selected explicit domain modeling and structured owner-native queries; amended the proposed target before dependent structured projection changes.
