---
id: ADR-0237
title: Retain full native initialization outcomes through recorded execution
status: proposed
date: 2026-11-02
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-07, PS-07, PS-12]
blueprint: [§17, §21.1, §21.5]
review: docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md#f01
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Another initialization policy cannot be represented without losing original intent or attempt evidence.
verification: Generic native and recorded initialization retain all successful/failed attempts and original restoration; exact readback and policy-sensitive identity tests discriminate last-result projection.
---

# ADR-0237: Retain full native initialization outcomes through recorded execution

## Context

The generic authored operation projects initialization to its last numerical result or an input error. The recorded declared-case preparation similarly assumes a prepared solve. Plan64c C03 requires full success/failure history through both selected boundaries.

## Scope

Extend the existing native operation/preparation/outcome and recorded result owners for initialization. Keep existing scientific declarations and typed report/table projections. No parallel initializer, report store or retry engine is selected.

## Drivers

An initialization may need its stages before a valid original start exists. Earlier attempts and rejected stages remain inspectable, while failure is distinct from invalid input and no failed attempt can publish an accepted original state.

## Options

Reject last-result projection because it loses scientific meaning. Reject serializing runtime services or using a second report store. Select an initialization-specific prepared/outcome variant within the existing operation lifecycle, reusing its original report and generated table projections.

## Outcome

Retain source, selected case, actual initialization policy and consumed method controls in the prepared operation. A purpose-specific framed identity includes stages, continuation/discrete policy and execution limits; it neither hashes serializer bytes nor changes existing frame meanings. Persist complete attempts, failures and committed original state through the current protected result owner. Version changed durable operation interpretations explicitly.

The selected recorded route is the native prepared initialization operation and existing canonical run/results lifecycle. The persisted StudyOperation protocol8 descriptor does not acquire an initialization policy by inference: converting a prepared initialization into that study descriptor, or preparing the historical `DeclaredCaseInit` descriptor, refuses. Extending the study wire protocol remains a distinct versioned change. Retained child results keep their own execution identity and original attempt; parent projections assign the actual initialization report ordinal so multiple child attempts cannot collide in recorded tables.

### Consequences

Migrate preparation, dispatch, completion, result projection, replay and readback together. Regenerate frame declarations through the owning generator route. Unsupported historical interpretations refuse rather than invent a report.

### Compensating controls

Preserve original assessment and restoration, exact parent/source identity, native cancellation/drain and diagnostic visibility. An initializer report does not establish a qualified physical candidate or supported expanded process population.

### Confirmation

Tests cover successful multi-attempt history, intended staged failure, unchanged specification, native/recorded readback and changed-policy identity. No passing evidence is claimed here.

## Pros and cons

The existing lifecycle gains truthful initialization outcomes at the cost of coordinated variant and durable-contract migration. A narrow explicit variant avoids a universal new task dispatcher.

## More information

Plan64c C03 owns execution and evidence; ADR-0236 owns scoped product permission. Formal ADR status remains proposed.

## Status history

- 2026-11-02: Proposed before recorded initialization contract changes during authorized Plan64b/64c execution.
