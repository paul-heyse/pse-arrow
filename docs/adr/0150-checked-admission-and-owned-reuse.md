---
id: ADR-0150
title: Admit checked occurrences and share role-typed owned products
status: proposed
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-06, DP-01, DP-04, DP-09, PS-01, PS-10]
blueprint: [§4.6, §5.3, §7.7, §14.3, §14.4, §18.8, §22]
review: docs/design_review/reviews/design_review_admission-reuse-and-generated-boundaries_2026-10-01.md
evidence: Implemented
supersedes: []
superseded-by: null
revisit: An admitted dependency or live allocation cannot be represented by its existing occurrence, identity or product owner.
verification: Plans 25h and 25i focused occurrence/context, framing, allocation-lifetime and counted-reuse controls; integrated storage and performance qualification remains 25k.
---

# ADR-0150: Admit checked occurrences and share role-typed owned products

## Context

Plans 25h and 25i address reparsing and ambient admission authority alongside incomplete reuse identities and unaccounted value rebinding. Their joins amend blueprint §4.6, §5.3 and §14.3 without creating a second source or mathematical authority.

## Scope

Checked expression occurrences, explicit validation context, canonical role-typed framing and immutable prepared-product ownership. Remove obsolete raw compiler and requirement-planner bridges; retain actual effect/configuration/allocation policy, physical/source enforcement and selected-view structural assessment.

## Drivers

Identical mathematics may appear at different source locations; binding changes must coexist rather than overwrite a singleton. Changing a consumed provider, import, physical definition or admission limit must invalidate reuse. Eviction and caller abandonment cannot release ownership while products or native work remain live. These responsibilities admit isolated counter, cycle and lifetime tests.

## Options

Retaining text-first occurrence lookup, process-global validation installation and package-local fixed-count LRUs preserves hidden authority and coarse reuse. A universal admission/cache/scheduler framework adds unrelated policy. Select existing checked occurrences and composition roots, local Salsa queries, DataFusion byte-bounded service retention and the current native supervision model.

## Outcome

Checking owns syntax, expression role, declaration/document-relative source origin and bound context once. Specialization consumes occurrence keys and constructs synthetic ASTs directly. Validation uses its immutable registry/planner/function/configuration owner; local contexts refuse obligations needing missing runtime semantics. Memo slots build outside the global map lock and refuse recursive or concurrent construction cycles with typed keys.

The identity owner supplies canonical framed serialization: NaN payloads canonicalize; signed zero and infinities remain distinct. Source revision, admitted closure, semantic body, prepared view, binding, profile, lineage request and operational job have distinct types. Changed preimages use new frame versions; historical digests are never recomputed. The operational attempt field changes through ADR-0146's preserving V6-to-V7 transition, retaining its recorded bytes and their historical frame provenance. Historical or unknown-frame digests remain typed recorded evidence; they cannot satisfy a current operational-job/cache key without proof of the required frame. Verification under an old frame remains distinct from computing a new identity.

Prepared products separate immutable mathematics, revision provenance and bound values, each retaining its actual allocation owner. Local tracked queries use complete admitted dependencies and immutable binding inputs. Service caches retain owned immutable admission/body/view products, never Salsa handles, mutable workflow packages, evaluators, cancellation or native workers. Revision publication always supplies its current attribution. Profile panic is attributable and never silently replayed; only unstarted work may move after scheduling failure.

Mathematical purity does not mean effect-free cache infrastructure: the service attachment may
look up owned bodies, count hits and transfer allocation leases. Attachment is selected once before
publication. A private compiler-issued body witness establishes the complete key and physical
closure; callback results cannot mutate that meaning. Transient allocation refusal unwinds Salsa
cancellation and returns its typed cause without memoizing refusal. Focused retry and foreign-body
controls establish this boundary. Operational hashes include explicit idempotency scope or the
submitted run/attempt/study occurrence; identical scientific requests can name different submissions.

The V7 transition also normalizes six exact PostgreSQL 18 NOT NULL names created with a `1`
suffix during immutable V5 table recreation. Verification admits only their exact table, kind and
definition before V7 renames them atomically; current layout verification requires canonical names.
No historical migration checksum or digest is changed.

### Qualification correction: checked-member attribution

**Proposed, 2026-10-02:** repeated PFR bodies expose an occurrence-specific validity member
inside the normalized body identity. Normalize that target to a body-local checked-member
token while retaining the complete range, physical types, authored annotation and consumed
context. Each instance owns a total, unambiguous token-to-actual-member binding, including
checked computed members. The map belongs to current binding/view identity and retained
accounting, not mathematical identity. Changed body and case preimages use new current
frame versions; historical frames and digests keep their bytes.

The existing instance-error attribution owner binds only typed validity member fields in
the complete retained diagnostic, recursively through nested causes. It preserves quantity,
declaration and provenance identities, source annotations, observations, classification and
recoverability. Missing or ambiguous binding refuses before execution; exhaustion never
drops a check. This separates reusable mathematics from actual attribution without dropping
semantic context or introducing a second diagnostic authority. The bounded Plan 25k review
assessed this contract before implementation. Verification must reuse one checked body for
distinct actual members, including computed targets, preserve each nested diagnostic, and
invalidate reuse when a range or physical type changes.

### Consequences

The public requirement-planner capability and raw compiler front door are removed directly. Actual provider effect/settings/byte policies and per-view structural witnesses remain. Ownership precedes broader retention; generated consumers migrate with changed producers. New cache entries retain positive overhead even when most payload is shared.

### Compensating controls

Compare final incremental admission with clean loading, test occurrence-specific attribution, exercise recursive and cross-worker cycles, freeze frame vectors, and count A/B/A/selective reuse. Retain escaped products through eviction/workspace rotation and check final reservation release. Preserve I4 completion-owned dispatch guards.

### Confirmation

**Implemented/Tested, 2026-10-02:** [25h Verification](../plans/25h-authoring-and-admission-ownership.md#verification) and [25i Verification](../plans/25i-identity-reuse-and-resource-ownership.md#verification) own the targeted force-validation commands, zero-failure baseline, composite repairs and limits. They cover checked occurrences and explicit contexts, canonical frames and role refusal, selective body/data/physical-context reuse, escaped allocation owners, profile dispatch and the preserving V7 transition. The original review assesses the proposed target; its dated follow-up independently assesses the implemented cache witness and mathematical-purity boundary. Counter and lifetime controls do not establish performance or total RSS bounds. Integrated qualification remains 25k; this record remains proposed pending its decision PR.

## Pros and cons

Explicit owners improve change locality and permit isolated admission/reuse tests. The coordinated Rust API cutover and preserving operational migration increase integration work.

## More information

[25h](../plans/25h-authoring-and-admission-ownership.md), [25i](../plans/25i-identity-reuse-and-resource-ownership.md), [series coordinator](../plans/25-design-remediation.md), ADR-0146 and ADR-0148.

## Status history

- 2026-10-01 — proposed before the integrated admission, identity and ownership change.
