---
id: ADR-0087
title: Close Plan 14 through local Linux qualification
status: accepted
date: 2026-09-24
deciders: [paul-heyse]
level: decision
principles: [DP-22, DP-23, PS-10]
blueprint: [§0.1]
review: docs/design_review/reviews/design_review_m22-local-qualification_2026-09-24.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A release or supported deployment requires another platform, distribution artifact or remote acceptance matrix.
verification: Plan 14 functional/performance architecture-acceptance receipts, eleven independent gate decisions, just adr-lint and just docs.
---

# ADR-0087: Close Plan 14 through local Linux qualification

## Context

The maintainer explicitly redefined M22 formal closure as local qualification.
Remote CI and Windows/macOS acceptance are not current priorities. Case rebuild
measurements must preserve the recently implemented Rust compiler caches.
The final maintainer clarification prioritizes proving the functional targets and
closing the design milestone, using the already reported full workspace pass and
the selected native/Python profiles. The upstream `proc-macro-error2` warning is an
accepted dependency limitation and requires no repair.

## Scope

Amend blueprint §0.1 for Plan 14 completion only: local qualification and independent
review authorize local decision acceptance and architecture reconciliation. The usual
PR/merge requirement is not a prerequisite for this milestone's local acceptance.
Remote branch protections, future release gates and other platform claims are unchanged.

## Drivers

Establish scientific and runtime truth on the actual selected environment without
conflating platform/release qualification or compiler-build cost with simulator behavior.

## Options

Require the full remote matrix: rejected by the maintainer for this milestone.
Treat local unit success as completion: rejected because Q01–Q18 and independent
scientific/runtime/design evidence remain mandatory.

## Outcome

Close Plan 14 only when the complete local Linux target, required repository checks,
case measurements and independent G1–G8/PS-G1–PS-G3 decisions qualify. Apply local
ADR/blueprint changes under this explicit scoped authorization. No merge or remote
CI result is implied. Benchmark process-case preparation and rebuilding, never clean
Rust builds; reuse stable checkout/target paths and compiler caches.

### Consequences

Windows/macOS, wheel/sdist, remote CI and release qualification remain separate work.
Existing acceptance tooling retains explicit required/advisory distinctions.
Release-profile tests/doctests, coverage, exhaustive feature powersets and dependency/
unsafe inventory campaigns remain available on demand; they are outside this local
design-stage closure. Default workspace tests, selected native profiles and repository
contract checks remain required. No failed functional target is waived.

### Compensating controls

Require complete exact witnesses, current source/native evidence, zero required
failures, independent reviewers, truthful supported profiles and immutable accepted
ADR arguments. Review findings cannot be waived by performance or platform scope.

### Confirmation

The M22 packet names complete functional and performance evidence, independent reviews,
remaining unsupported behavior and the final local document/decision checks.

## Pros and cons

Local closure is actionable and matches the selected target; it gives no evidence for
unexecuted platforms or distribution artifacts.

## More information

See [M22 execution](../plans/14-m22-execution.md). ADR-0034 continues to govern remote
repository operations; this milestone-specific authority does not alter its argument.

## M22 local qualification

**Tested and Measured:** the [M22 packet](../plans/14-m22-execution.md#verification)
records local Linux functional Q01–Q17, the 23 cached-development case-cost workloads,
zero required failures and retained-origin conditions. It distinguishes admitted memory
allowances from measured pool/RSS observations and excludes Rust build time.

The [independent final review](../design_review/reviews/design_review_m22-local-qualification_2026-09-24.md)
accepts the relevant scoped contracts with no open MUST finding. Companion runtime,
scientific and claims reviews cover G1–G8 and PS-G1–PS-G3. Blueprint revision 51 and
ADR-0087 govern local acceptance. Strict Clippy cleanup and release/remote/platform
qualification remain separate; no broader clean or empirical claim follows.

## Status history

- 2026-09-24 — proposed under explicit maintainer authorization for local closure.

- 2026-09-24 — accepted for local Linux M22 scope under ADR-0087 after independent final review; blueprint revision 51 reconciles the contracts. No remote or release qualification is claimed.

## Design-stage static boundary

The maintainer prioritizes functional target proof and forward design progress.
Strict workspace Clippy cleanup is separate work: the attempted run exposed
documentation and style findings across earlier implementation packets and is not
claimed clean. Normal compilation, functional tests, schema/governance tests,
Python quality, generation, ADR and book checks remain required. The full workspace
test run already includes governance tests; a second package-only governance run
is redundant and would build another dependency-feature graph. Ordinary merge
and release quality requirements remain unchanged.
