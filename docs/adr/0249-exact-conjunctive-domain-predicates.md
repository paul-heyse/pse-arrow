---
id: ADR-0249
title: Represent exact conjunctive domains with first-class predicates
status: proposed
date: 2026-11-06
deciders: [paul-heyse]
level: decision
principles: [AP-04, DP-01, DP-15, PS-05, PS-13]
blueprint: [§7.3, §7.5]
review: "not-required: bounded extension of the existing guarded-math predicate kernel within authorized Plan64/65 execution; focused independent source design assessment precedes activation"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A selected domain requires a predicate beyond a conjunction of ranges and nonzero exclusions, or a backend can consume an exclusion more precisely than its declared closed relaxation.
verification: Mixed range/nonzero conjunctions, equality and negation boundaries, tiny and large differences, original occurrence/order/scope identity, malformed references, interval discharge and uncertainty, interval-only relaxed export, and original design/rating exchanger comparisons plus protected scientific intake.
---

# ADR-0249: Represent exact conjunctive domains with first-class predicates

## Context

The original design exchanger's logarithmic-mean law declares positive temperature
differences and `a != b`. Exact arithmetic supports the equation and IBEX already supports
nonzero guards, but the projection stores only range constraints. Its Domain occurrence
therefore remains unrepresented and withholds output accuracy. Changing the law or dropping
the exclusion would conceal a representation gap.

## Scope

Supplement blueprint §7.3 and §7.5 with a conjunctive predicate vocabulary. Closed implicit
bounds remain range constraints. Backend closed-relaxation policy, mathematical authority,
root/accuracy evidence grades and physical acceptance remain unchanged.

## Drivers

Preserve complete original domain meaning and provenance across projection, interval
admission, curvature, presolve, native evaluation and result interpretation. Use the existing
native nonzero capability directly and keep one executable predicate owner.

## Options

Retaining closed-only projection refuses an available capability. Squaring the difference
adds avoidable overflow, underflow and work; a margin changes the domain. Splitting a Domain
into authored Require records misstates its source occurrence. A separate exclusion vector
duplicates conjunction handling. Select `Predicate::{Range(Constraint), Nonzero(NodeId)}`
shared by truth projection, obligation predicates and root eligibility; genuine disjunctions
remain unrepresented.

## Outcome

`ProjectedObligation` retains one original instance/source/kind/scope/argument/order occurrence
and one complete predicate conjunction. Require conditions normalize into that same predicate
representation; kind/argument retain provenance rather than causing a second native predicate.
Inequality becomes a nonzero predicate over the original difference. Exact-real projection
continues to govern arithmetic precision independently of predicate vocabulary.

Native IBEX lowers Nonzero to its existing kind-2 guard, with no square, margin or branch
selection. Interval admission establishes it only when the enclosure excludes zero, rejects
the zero singleton, and otherwise withholds. Interval-only exports explicitly mark omitted
nonzero predicates Relaxed; exact-only consumers refuse that loss. Conditional predicates
and unconsumed derivative requirements retain their existing admission distinction.

### Consequences

All predicate consumers must match both variants, validate references, account for storage,
and preserve complete conjunction equality when establishing unconditional coverage. Source
identity frames discriminants and payloads, all occurrence metadata and derivative order.
Version the exact-program payload tag to v2; old evidence is not silently rebound. This
uses the existing framed hashing policy and does not change the hash algorithm.

### Compensating controls

Keep source-attributed unavailable results for unsupported predicates, genuine disjunctions,
opaque dependencies and unresolved interval zero crossing. Checked preparation, deduplication,
work/cancellation ownership and native resource limits remain. No closed relaxation becomes
original-domain or scientific permission.

### Confirmation

**Proposed:** the verification population above plus actual original scientific comparisons.
Focused source assessment establishes the existing native interface; it does not establish
executed behavior. Plan64h owns scientific acceptance; Plan65e/65h own integration and its
evidence boundaries. This record remains proposed until its separate decision route changes it.

## Pros and cons

The sum type expresses the needed domain directly and reuses existing library execution.
It requires coordinated source/API/identity migration and explicit fidelity handling by
interval-only consumers; it does not add general Boolean or disjunctive solver machinery.

## More information

- [Predicate and factorable owner](../../crates/pse-math/src/factorable.rs).
- [Existing IBEX guard handling](../../crates/pse-backend-native/native/root_isolation.cpp).
- [Scientific acceptance](../plans/64h-reference-coverage-and-qualification.md#source-anchored-output-assessment--november-6-execution).
- [Library mechanisms and execution owner](../plans/65e-scientific-portability-and-execution.md#original-arithmetic-preparation--november-6).

## Status history

- 2026-11-06 — proposed before implementation from the retained logarithmic-domain refusal.
