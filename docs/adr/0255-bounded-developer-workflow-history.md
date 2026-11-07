---
id: ADR-0255
title: Retain selected developer workflow history with bounded SurrealDB custody
status: proposed
date: 2026-11-06
deciders: [paul-heyse]
level: decision
principles: [AP-04, DP-02, DP-09, DP-19, DP-21]
blueprint: [§6.13, §21.7, §23.1]
review: "not-required: scoped pilot for derived developer observations within the existing observability contract"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An approved consumer needs a field beyond the bounded projection, or repeated ingest and expiry fails to stabilize physical allocation.
verification: Verify independent triage and current-inventory rerun after original executable release, publication retries and ambiguous acknowledgements, role isolation, expiry with holds, physical reclamation across restart, and selected cohort restore.
---

# ADR-0255: Retain selected developer workflow history with bounded SurrealDB custody

## Context

Developers need to triage selected failures, rerun them against a current build, and compare
named cohorts after the original target and working files are gone. ADR-0252 serves queries
over an explicitly supplied finite population. Continued discovery of selected invocations
across working-file release activates its persistence revisit condition.

## Scope

Extend blueprint §6.13, §21.7 and §23.1 with a scoped pilot for derived developer history.
The native report remains the original verdict. ADR-0252 continues to govern finite queries
over supplied members. Scientific runs, published data, active execution and control retain
their separate owners.

## Drivers

Give each retained fact a consumer, provenance and lifetime. Allow an independently verified
projection to outlive the expensive inputs it describes. Keep the runner usable during history
outages and allow operators to verify both logical expiry and physical storage reuse.

## Options

The finite reader stays suitable for supplied cohorts. Using it for discoverable history
would require another custodian for the files and their holds. A full-result archive would
duplicate working data. Select a bounded SurrealDB projection so indexed discovery, expiry and
atomic writes can share one lifecycle. Pilot separate credentials and database selection on
the existing server. Use a separately managed instance if credentials, capacity or recovery
cannot be isolated there.

## Outcome

The original native adoption owner authenticates the source before publishing the fields
needed for a named purpose. Give each record a deterministic original invocation and fact
identity. Store content commitments, action, verdict, coverage and omissions, and only
eligible selected test identities or cohort facts. Use a schema owner with typed declarations
and indexes for the bounded reads and expiration scans. Publish related rows atomically.
An identical retry returns the committed publication. Conflicting facts refuse and an
uncertain acknowledgement is resolved against the original identity before retry.

### Consequences

The developer-history owner sets finite windows and bounded read and mutation populations.
The pilot measures record sizes and read patterns before setting ordinary-use windows.
Named cohorts carry an explicit purpose and expiry. Raw members stay with their current
custodian and carry an independent hold and release condition. A history reader can supply
verified original selection facts to the current inventory resolver. The resolver retains
admission and new execution authority.

### Compensating controls

Separate publisher and reader credentials from scientific data access. History faults carry
their own outcome and bounded retries. A fault neither changes the native verdict nor extends
ordinary working-file retention indefinitely. Track logical expiry separately from allocated
bytes and qualify a supported reclamation path before activation. Use selected portable
exports for cohorts that must survive loss of the pilot store.

### Confirmation

Plan67b verifies independent readback, role isolation, matching and conflicting retries, an
uncertain acknowledgement, and history outage. It exercises selection and rerun after old
executable release. It verifies bounded expiry with a protected hold, physical growth through
repeated ingest and restart, and restore of a selected cohort.

## Pros and cons

- The same structured records support triage, comparison and finite expiry after a target clean.
- The pilot adds storage and credential ownership, plus reclamation and recovery obligations.

## More information

[Plan67b](../plans/67b-invocation-custody-and-developer-history.md) owns pilot evidence.
[ADR-0252](0252-ephemeral-developer-evidence-relations.md) owns the supplied finite-query contract.

## Status history

- 2026-11-06 — proposed.
