---
id: ADR-0216
title: Keep project ownership canonical and evidence independently derived
status: proposed
date: 2026-10-22
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-06, DP-09]
blueprint: [§20.5, §20.6, §24.1]
review: "not-required: New relation families within existing canonical and host authority decisions; Plan51 foundation assessment retains topology and payload contracts."
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A required standalone editable deployment or atomic byte/catalogue operation cannot be served by the selected topology.
verification: Project head-CAS and exact-settlement controls; isolated evidence generation, permission and database-unavailable controls in Plan51a.
---

# ADR-0216: Keep project ownership canonical and evidence independently derived

## Context

Plan51 adds persistent engineering project interaction and selected cross-run evidence queries.
Existing source heads, protected results and host invocation/resource owners already establish their authority.

## Scope

New project/research metadata and derived evidence relation families within blueprint §20.5–§20.6
and §24.1. Retain the shared SurrealDB/MX service, sealed payloads and existing Python boundary.

## Drivers

A branch must select an independent canonical source head. An evidence index must remain disposable
without altering terminals, scientific results or resource custody. Public Rust/Python and operator
consumers must use the same typed guarded operations.

## Options

Extend the canonical catalogue for project/research metadata and add an independently attested
tooling database on the selected service. Direct KV/MX and embedded alternatives were assessed
in Plan51 and are not selected; they transfer consumed query, lifecycle and isolation obligations.
A second generic storage facade or a database-owned host registry adds no needed capability.

## Outcome

Project branches reference distinct canonical source problems; the existing source head remains
authoritative. Project publication composes its guards and receipt with source activation.
Evidence uses separately generated installation/codec identities, database-scoped actors and
complete imported generations. It cannot settle source authorities or grant raw artifact lifetime.

### Consequences

New families require registry declarations, generated contracts and actual consumer integration.
Tooling opening cannot reuse scientific opening or native receiver requirements. Bulk staging and
scientific work stay outside short metadata transactions.

### Compensating controls

Original operation identities, head/generation checks, protected readers, complete response checks
and explicit initialization preserve existing failure semantics. Original host inspection remains
usable when the service or native product is unavailable.

### Confirmation

Interface-checked against the current source activation, generated schema, SDK authentication and
transport lifetime owners. Implementation and targeted test evidence are owned by Plan51a;
this proposed decision does not establish assembled scientific or performance qualification.

## Pros and cons

Retains consumed native database invariants and independent tooling inspection; requires explicit
integration of new project ownership and a second database interpretation on the same service.

## More information

[Plan51a](../plans/51a-storage-foundations-and-project-ownership.md) owns implementation;
[Plan51](../plans/51-engineering-workspaces-and-evidence.md) owns package state and opportunity disposition.

## Status history

- 2026-10-22 — proposed; maintainer authorized Plan51a execution.
