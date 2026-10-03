---
id: ADR-0153
title: Qualify nonlinear regime selection with library-owned root evidence
status: proposed
date: 2026-10-03
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-06, DP-09, PS-10]
blueprint: [§7.5, §14.3.2]
review: 'not-required: bounded kernel binding within the existing library-owned mathematics decision; focused independent design assessment accompanies Plan 25k'
evidence: Tested
supersedes: []
superseded-by: null
revisit: A requested criterion, tolerance or original domain cannot be represented by the validated interval library, or useful selection requires discovering a winner without a verified numerical proposal.
verification: Nonlinear selection controls distinguish a complete parameter chart with global exclusion from a local chart, multiple roots, unresolved coverage, boundary entry and stale inputs; the unchanged nested PR fixture exercises production lowering and IFT derivatives.

---

# ADR-0153: Qualify nonlinear regime selection with library-owned root evidence

## Context

Plan 25k's nested PR journey requires derivatives of minimum-score selection
across nonlinear residual regimes. A regular native iterate and a gap between
regime scores do not establish which root was selected within a regime. The
existing value-only restriction is correct; completing the journey requires
additional evidence rather than a higher solver budget.

## Scope

This adds a bounded validated-root kernel binding within library-owned
mathematics, amending blueprint §7.5 and §14.3.2. It preserves the authored
minimum-score meaning, physical bounds, independent inputs and original-space
qualification. The winning numerical proposal must have a validated regular
chart. Competing roots need not be enumerated: complete validated exclusion
establishes their strict separation from the winner’s authored score and tie band.

## Drivers

The compiler owns exact residual, eligibility and guard projection from the
existing factorable DAG. A native validated-interval library owns contraction,
root isolation and parameter charts. Generic regime selection consumes the
evidence and retains its existing score-gap, original residual, physical-bound
and First/Second IFT checks. Numerical starts remain numerical.

## Options

- Retain value-only nonlinear selection: sound, but cannot complete the authorized
  derivative journey.
- Promote a converged native root or local chart: inexpensive, but another
  eligible root outside that chart remains possible.
- Add authored branch anchors: possible separate semantics, but changes the
  current minimum-score operation and requires its own branch contract.
- Use IBEX validated interval evidence: selected. Exact-input isolation and
  parameter-neighborhood selection have separate contracts. Symbolica's scalar
  coefficient/root facilities do not establish the coupled PR system; Numerica's
  documented certified arithmetic scope does not include its logarithms.

## Outcome

First/Second nonlinear regime selection requires a parameter box containing the
evaluation inputs in its interior and a regular eligible winning root chart for
every input in that box. Complete exclusion must establish that no eligible root
in any other regime, or outside that chart in the winning regime, can beat or tie
the winner. Exact criterion and tolerance projection supplies a uniform winning
score upper bound and a conservative comparison band. Closed slabs cover the
winning chart's complement; whole domains cover rival regimes. A losing regime
may contain multiple or singular roots. Its numerical failure is neither empty
evidence nor dominance evidence. Point isolation alone never raises derivative order.

### Consequences

Projection retains exact binary constants or sound rational enclosures, strict
and nonzero guards, coordinate ordering and original physical bounds. An
unsupported or conditional guard is a refusal unless its full meaning is
represented. Restricting search using a proved eligibility predicate does not
alter physical bounds. Native undefined arithmetic cannot by itself discharge
a crossing guard obligation. Competitive roots, boundary ambiguity and incomplete coverage remain refusals
even when an accurate root is available. A tolerance must have a validated finite
nonnegative enclosure over the compared domain; inability to enclose it refuses.

### Compensating controls

Proof work has finite time, cells and retained bytes, and shares the outer
absolute deadline, cancellation and admission. Repeated evaluations consume
that deadline's remaining time; they cannot renew the enclosing attempt's clock.
Late native evidence is rejected before retention, with time exhaustion distinct
from cancellation. Evidence belongs to its exact program and bounds;
neighborhood evidence may only be reused within its certified parameter box.
A single worker-owned certificate may retain the exact immutable alternative
programs, identities and physical bounds; higher orders and changed source,
bounds, winner or parameter scope require new evidence. Its finite retained
extent belongs to the same admission as the worker, with no global proof cache. The binding statically embeds PIC libraries with hidden C++ symbols;
the validated LP ABI cannot interpose with SCIP's separately embedded SoPlex.
Exceptions remain inside the C++ adapter. Failure does not silently switch backend, clip bounds or weaken
physical acceptance.

### Confirmation

**Implemented:** exact criterion/guard projection, whole-selection IBEX 2.9.1
competitive exclusion, bounded worker-owned charts and shared absolute execution
scopes are integrated. The contract requires uniform local existence/uniqueness
and complete exclusion of the eligible competitive complement.

**Tested, 2026-10-03:** local Linux
`just unit-native-package pse-backend-native pse-backend-native/native-solvers 'test(root_isolation::)' --test-threads 1`
passed all 16 controls with explicit force-validation. They exercise global
selection, noncompetitive multiple/singular rivals, singular winners, ties,
boundaries, exact constants and no independent parameters. The largest-first
bisector retains the same proof requirements and original domains.
`just unit-package pse-math 'test(implicit) | test(factorable)' --test-threads 1`
passed 47 controls, including expired outer scopes, capped repeated evaluations,
late native success and late positive proof retaining neither chart nor branch.
These bounded controls do not establish full scientific qualification. The owning
Plan 25k retains the unchanged nested PR journey and integrated closure obligations;
ADR status does not certify that journey.

## Pros and cons

The selected route preserves authored meaning and uses existing validated
mathematics. Its cost is potentially substantial global proof work, and some
otherwise solvable requests can remain uncertified under finite budgets.

## More information

[Plan 25k](../plans/25k-integrated-qualification-and-closure.md) owns execution,
qualification and remaining findings. IBEX's [solver documentation](https://ibex-team.github.io/ibex-lib/solver.html)
describes validated coverings and unresolved boxes; the initial binding targets
tag `ibex-2.9.1` and bundled FILIB interval arithmetic. Enduring contracts belong
to [mathematics and compilation](../authoritative_design/sections/mathematics-and-compilation.md).

## Status history

- 2026-10-03 — proposed.
