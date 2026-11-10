---
id: ADR-0262
title: Use typed canonical operations and owned schema transitions
status: proposed
date: 2026-11-09
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-06, AP-07, DP-09, DP-24]
blueprint: [§4.7, §20.5, §20.6.11]
review: docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A selected operation needs cross-transaction dependency protection or a retained source cannot satisfy admitted transition postconditions.
verification: Plan72b original migration/recovery and Plan72c operation, readiness and SSI controls with independent data expectations and enclosing-write rollback controls.
---

# ADR-0262: Use typed canonical operations and owned schema transitions

## Context

The maintainer authorized Plan72b/72c execution after accepting RC01/RC02. Existing
procedures and typed envelopes already own canonical effects. Implicit isolation,
JSON policy carriers and broad publication guards obscure their actual contracts.
Current maintenance re-epochs the current schema rather than migrating retained values.

## Scope

Amend blueprint §4.7, §20.5 and §20.6.11 for typed operations, shared pure policy
and owned transitions. Scientific algorithms, external-effect authority, original
acknowledgements and validation-only ordinary opening keep their existing owners.
The record remains proposed; Plan72b/72c own implementation and Plan72i receiving.

## Drivers

One semantic intent yields one meaningful typed result. Exact source/target meaning,
original identity and physical exclusion survive interruption. Native predicate tracking
replaces concurrency-only blanket writes within one transaction. Independent scientific
expectations remain necessary when representations change.

## Options

Retaining implicit isolation and JSON adaptation minimizes edits but preserves repeated
interpretation and contention. Transaction-access extensions hide relational dependencies.
Select visible SQL acquisition/publication, explicit serializable mutations, read-only
snapshot observations and owner-derived native carriers around the pure Rust policy.

Prefer fresh reconstruction for explicitly disposable state; named native conversion for
eligible field-local transformations; bounded application backfill otherwise. Delegate
frozen history/catch-up to the prepared SurrealKit CLI instead of duplicating its journal
or adding its SDK to the production closure. Expected simplification needs no benchmark.

## Outcome

Typed operations declare isolation and authority. Installation retains its snapshot
exception; custody acquisition is a mutation. Serializable read-set coverage may replace
concurrency-only guards after actual positive/negative/range races qualify. Semantic,
cross-transaction, issuer, lease and original-request fences remain.

Semantic owners emit readiness projections. Native and WASI callers share generated
adapters through the existing module library. Aggregate admission precedes nested owned
decoding. Exact native U64 encoding and checked wire widening preserve numerical meaning.

The maintainer clarified that existing scientific results need not survive the design-phase
cutover. Select fresh reconstruction of explicitly disposable/reproducible state into the
clean v12 target, with fresh issuer and credentials. Authored source inputs remain outside
reset authority. Do not build a v11 compatibility reader or retained-results backfill estate
for this cutover. Native migration mechanisms remain available for separately admitted useful
transitions; they do not delay current fresh-state adoption.

Transition intent binds issuer, predecessor, frozen effective inputs, original/target
module artifacts, effects and preservation policy. The actual lifecycle owner closes and
drains ordinary participants/server before isolated maintenance. Exclusion lasts through
index gaps and activation. Ordinary readmission follows physical handoff. Supplier history,
data conformance, ready indexes and activation remain distinct facts. Unknown replies
reconcile the original operation before replay. Fallbacks cannot silently repair data;
preserve originals and independently compare membership and values. Reverse DDL cannot
restore discarded information.

### Consequences

Move consumers and delete replaced framing, JSON adapters and concurrency-only guards
together. Admit complete historical mappings explicitly; one scalar fixture is not a
whole-schema migration. Drain captures through original recovery rules before retiring
those definitions and retain exact-request tombstones.

### Compensating controls

Retain runtime rollback/admission/physical-release patches and the constrained definer
control API. Normal RETURN/HTTP refusal does not imply rollback. Retry settled typed
conflicts under one original clock. No arbitrary configured store is reset or migrated.

### Confirmation

Interface-checked against selected v4 source, pinned SurrealKit rc.1 and native owners.
The integrated functional review supplies architectural reasoning. Executed evidence
belongs to the owning plans; ADR publication does not claim implementation acceptance.

## Pros and cons

The target removes repeated interpretation and concurrency work while keeping relational
dependencies visible. Complete cutover/recovery integration remains necessary: native
migration completion alone does not establish application readiness.

## More information

[Plan72b](../plans/72b-schema-evolution-and-deployment.md),
[Plan72c](../plans/72c-canonical-operations-and-policy.md), and the
[functional review](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md).

## Status history

- 2026-11-09 — Proposed before maintainer-authorized Plan72b/72c implementation.
