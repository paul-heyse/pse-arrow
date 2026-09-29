---
id: ADR-0120
title: Relax a provider to its declared, enforced output envelope
status: accepted
date: 2026-09-28
deciders: [paul-heyse]
level: decision
principles: [AP-02, DP-03, DP-08, PS-02, PS-10]
blueprint: [§7.5, §9.4]
review: docs/design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0120
evidence: Tested
supersedes: []
superseded-by: null
revisit: A provider output is observed outside its declared envelope, or a certify-route model needs an enclosure that depends on the provider's inputs rather than constant output intervals.
verification: Items 1–4 are settled by the crate tests provider_envelope_is_checked_against_the_contract (pse-kernels), relaxed_rows_enclose_evaluator and unavailable_objective_reported (pse-math), run by the Plan 22 G4 packet (commit 7d0f8bf1, merge 42f5a83f) and not rerun for this record. Items 5–7 are settled by extending provider_envelope_is_checked_against_the_contract with (+inf, +inf) and (-inf, -inf) intervals, and by a test in which a worker returns an output outside its declared envelope and the evaluation fails as a contract error.
standard: core-3.1/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s12]
---

# ADR-0120: Relax a provider to its declared, enforced output envelope

## Context

[ADR-0105](0105-scip-factorable-backend.md) item 1 makes a provider output an `Aux` "within
enforced envelopes" in the factorable projection, so that the rows depending on it are
`Relaxed`. It does not say where an envelope comes from, what it promises, or who checks
it. Plan 22 G4 (commit `7d0f8bf1`) added a minimal typed kernel extension for the
declaration: `ProviderFactory::envelope` and `Registration::envelope`. The G6 kernel-gap
note asks for a short record of that contract. No production provider declares an envelope
today.

## Scope

A kernel contract within ADR-0105, which is not superseded. It binds what a provider
envelope promises, how it is checked, and how the factorable export consumes it. It governs
blueprint §9.4 (the external-function contract) and §7.5 (the factorable projection).
Implicit blocks, which export their residuals instead, are unchanged.

## Drivers

- **PS-02 and DP-03.** An envelope is a validity claim about a provider's outputs. A soundness-critical claim needs an enforcement point and an observable rejection; documentation is not one.
- **PS-10 and ADR-0105 item 2.** A `global_bound` or `proven_infeasible` from a `Relaxed` export is only as sound as the relaxation. A false or empty envelope turns it into a false claim.
- **DP-08.** Replacing a provider by a bounded auxiliary is a named transformation with a declared equivalence: a relaxation, never exact.
- **AP-02.** The kernel host (`pse-kernels`) owns the provider contract. The projection consumes only checked intervals, never provider internals.

## Options

| Option | Assessment | Selection |
|---|---|---|
| No declaration: a provider output is always a free auxiliary | Sound, but an unbounded auxiliary inside a nonlinear term refuses the export, so certification never applies to such a model | Rejected |
| An envelope from the registry's quantity declaration or the case bounds | A quantity kind carries no output range, and case bounds constrain variables rather than provider outputs | Rejected |
| An input-dependent enclosure (interval or estimator callbacks) | This is SCIP's user-expression-handler route, which ADR-0105 rejected | Not now; revisit trigger |
| **Constant output intervals declared by the provider factory, checked by the host, enforced at evaluation** | Small, typed, and sound when enforced | **Selected** |

## Outcome

1. **Declaration.** `ProviderFactory::envelope()` returns `None` or one interval per output, in output order.
   - `None` is the default and means that evaluation enforces no envelope.
   - An interval promises that every successful evaluation of that output, anywhere in the provider's admitted domain, lies inside it.
   - A provider keeps that promise by construction, or by rejecting the trial with a typed trial or envelope error. A provider that cannot guarantee a range declares none.
2. **Checked declaration.** `Registration::envelope()` checks the declaration against the admitted contract: one interval per output, no NaN endpoint, and lower ≤ upper. Anything else is a `ProviderError::Contract` refusal.
3. **Consumption by the factorable export only.**
   - On a factorable route, the runtime collects each provider's checked envelope into `FactorableRequest.envelopes` before any worker exists (`math/solves.rs`).
   - An unconditional provider call without an implicit definition becomes one auxiliary per output, bounded by its interval. A call inside a branch region stays unbounded.
   - The dependent rows and objective are `Relaxed`, never `Exact`. They support a dual bound and an infeasibility conclusion, never a solution claim, and every candidate is still qualified by the evaluator (ADR-0105 item 2).
   - A bounded auxiliary removes the missing-bound refusal for spatial branching, and makes an objective that depends on it `Relaxed` rather than `Unavailable`.
4. **Identity.** Every interval's bits enter the factorable program key, which enters the step's preparation identity.
5. **Real intervals** (review F04). Each interval also contains a real number: lower < +∞ and upper > −∞. The check in item 2 admits (+∞, +∞) and (−∞, −∞), which are empty over the reals and would make a `Relaxed` export no relaxation at all.
6. **Enforcement at evaluation** (review F04).
   - The host checks every successful value evaluation of a provider that declares an envelope.
   - An output outside its interval is a `ProviderError::Contract` naming the output. It is never a trial rejection, because it shows that the declaration is false, not that the trial point is inadmissible.
   - The check detects a broken promise wherever one is exercised. It cannot prove the promise globally, which stays the provider's obligation.
7. **Identity owned by the host** (review F04). `Registration::configuration_key` frames the checked envelope itself. Today the trait's documentation asks each factory to include it, and nothing checks that it does. Once item 6 makes an envelope change evaluation outcomes on every route, the provider's identity must carry it.

### Consequences

- Certification applies to models whose provider outputs enter nonlinear terms, provided each provider declares an envelope. The resulting bounds are only as tight as the constant intervals.
- Items 5–7 are small changes in `pse-kernels`: the interval check, a checking wrapper around the worker, and the configuration key. They belong to the G6 kernel gaps, which the solver scope packet owns; the packet records when they are scheduled.

### Compensating controls

- The relaxation-soundness rule of ADR-0105 item 2: evaluator qualification of every candidate, and no solution claim from a `Relaxed` export.
- The typed contract refusals of items 2, 5 and 6.

### Confirmation

- **Tested:** items 1–3.
  - `provider_envelope_is_checked_against_the_contract` covers the default, an accepted declaration, and the arity, reversed-interval and NaN refusals.
  - `relaxed_rows_enclose_evaluator` covers the bounded auxiliary, the relaxed row and the absence of missing bounds.
  - `unavailable_objective_reported` covers an enveloped objective that becomes `Relaxed`.

  These crate tests ran in the Plan 22 G4 packet; this record did not rerun them.
- **Implemented, not tested end to end:** the runtime collection in `math/solves.rs`, and item 4 (the program key and the preparation identity), traced in source.
- **Proposed:** items 5–7, from the review's finding F04.

## Pros and cons

Constant intervals are the smallest contract that makes provider-bearing models certifiable.
They are sound only if enforced, which is what items 5–7 add. An input-dependent enclosure
would give tighter bounds, at the price of the interval and estimator callbacks ADR-0105
rejected.

## More information

- [Change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0120), finding F04.
- G4 commit `7d0f8bf1` (merge `42f5a83f`); the G6 kernel-gap note in the [main execution packet](../plans/22-solver-capabilities-execution.md); the [solver scope packet](../plans/22-solver-scope-execution.md), which owns items 5–7.
- Sources: `crates/pse-kernels/src/lib.rs` (`ProviderFactory::envelope`, `Registration::envelope`), `crates/pse-math/src/factorable.rs` (`FactorableRequest.envelopes`), `crates/pse-runtime/src/math/solves.rs`.
- Related: ADR-0084, ADR-0105, ADR-0119.

## Status history

- 2026-09-28 — proposed and accepted under the maintainer's authorization of Plan 22's full scope, and of the solver scope packet's improvements (2026-09-28), after the [change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0120) returned Accept (author review; items 1–3 Tested, item 4 Implemented, items 5–7 Proposed). Finding F04 was corrected in this record before acceptance.
