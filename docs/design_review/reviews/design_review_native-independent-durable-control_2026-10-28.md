---
title: Native-independent durable control
date: 2026-10-28
tier: design
purpose: target
decision: Accept
doc_role: review
reviewer: Independent design-reviewer, coordinator publication
doc_owner: docs/plans/57-library-leverage-and-correctness.md
standard:
  core: "3.4"
  efficient-architecture: "1.0"
  test-assessment: "1.2"
  process-simulator: "1.5"
---

# Native-independent durable control

## Assessment and scope

**Architectural fitness: Accept for the bounded Proposed target in amended ADR-0232.
Behavioral acceptance: not established.** This independent judgment permits implementing
browse/cancel/reconcile; it does not accept its eventual implementation or close R01/DA-046.

The source baseline is `2138a72b5`, including retained operations inheritance corrections
and 57j checkpoint. The review consumes the scoped production/test audits at `5e8aea0cc`
and principal review at `277eee026`, then the amended ADR-0232. No broad audit restarted.
The reviewer wrote no code and ran no tests, builds, services or measurements. This is the
coordinator's condensed publication of that independent assessment.

Scientific preparation, numerical algorithms, protected payload readers, subscriptions,
alternate backends and an independently installable distribution remain outside this slice.

## Why the selected boundary fits

Current NativeRuntime construction and package initialization acquire scientific dependencies
before compact access. Yet workbench selection consumes issuing-store/cursor identity,
activity, response storage and compact canonical selection. Moving these to a connection/control
owner removes R01's established dependency. Python carries intent, not scientific admission,
producer evidence, solver assessment or proof of physical drain. Local science and workers retain
one scientific interpretation and the existing performing owners.

| Capability | Required owner |
|---|---|
| Browse/cancel/reconcile | Authenticated connection, operation authority, finite request/response policy |
| Hydrate retained scientific data | Exact selection/protection and actual decode/delivery custody |
| Prepare/execute/publish science | Shared admission, qualified producer and original-model assessment |

The compact selector preserves indexed predicates, finite pages and no scientific hydration.
One generated database selector serves public and retained Rust callers.

## Consequential contracts settled in ADR-0232

The amended decision resolves material gaps identified in its initial draft.

Current finish_study_cancel reconciles cancelled state by study key, not caller attribution.
The new command uses an unambiguous issuing-store/actor/operation tuple, immutable full intent,
and a result persisted atomically with the guarded decision. Same-command replay returns it;
changed-intent reuse refuses. Compact receipts, including authenticated domain refusals,
survive study retirement for the database lifetime. Restore changes issuer identity. Absence
cannot prove an in-flight command will not commit.

The effective server checks namespace/database scope, then disables ordinary permissions
inside API bodies. Explicit enabled actor and operation grants must precede state and receipt
access. Native entry requires trusted system identity; a supplied actor cannot bypass it.
Ordinary clients cannot acquire privileged writer credentials or provision their own grants.

Lazy root exports alone are insufficient. Generated values import native identity helpers,
codec imports native header validation, and version fallback calls native_version(). The
corrected target removes these from the control closure while retaining artifact/registry
checks and Arrow initialization at the actual _build/scientific gate, including private imports.
Unavailable distribution metadata must not load native code or fabricate a version.

The original clock covers conversion, admission, delivery and the guarded decision. HTTP phase
timeouts are not a total deadline. Async cleanup owns local response/client completion;
cancellation, receipt acknowledgement, remote rollback and native drain remain distinct.

## Library choice and architectural judgments

HTTPX2 2.16.0 raw streaming and AnyIO 4.15.1 cancellation address this boundary. Released SDK
whole-response buffering lacks the public admission hook; development APIs cannot be attributed
to that release. This is a bounded choice, not general rejection of the SDK. Count raw bytes
before decoding, enforce finite document shape, refuse compression/redirects and preserve exact
integers. A byte cap alone does not prove all intermediates or decoded allocations are prepaid.
Request-size admission precedes server parsing. Cancellation and its receipt share one transaction;
unknown delivery grants no fresh mutation. Existing generations and worker drain owners remain.

AP-01–AP-07 are satisfied by the Proposed target: control/science separation, explicit stable
contracts, shared transition owners, distinct authorities, structural grants/identity, local
testability and compact bounded execution. At document-assessment strength the target supplies
mechanisms for G1–G8 and satisfies G9 without a SHOULD exception. Implemented validity, atomicity,
completion and library behavior remain unestablished. PS-G1–PS-G3 are not requalified.

## Decision and follow-up

Accept amended ADR-0232 at Proposed architecture strength. The actor/receipt lifecycle, actual
scientific gate and completion distinctions are required, not optional refinements. Review
RC01–RC03 are confirmed; this slice selects no RC04 expansion or additional exception.

Principal review S01/S03/S04/S05/S06 remain applicable. Effective-server tests must distinguish
actor denial from setup refusal, exact replay from state idempotence and acknowledgement from
drain. Completion needs native-loading-refused public use, migrated consumers, deleted forwarding,
and actual authorization/receipt/deadline/worker evidence. Focused controls cannot substitute
for those journeys. Coordinator owns disposition, K/L migration and J assembled acceptance.
Startup, memory, throughput and capacity benefits remain unmeasured.
