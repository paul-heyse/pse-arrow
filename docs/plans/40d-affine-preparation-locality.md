---
title: Original affine-row preparation with local dependency scope
status: done
date: 2026-10-14
adrs: []
review_sources: [docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md]
scenario_sources: [docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md, docs/design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md]
doc_owner: docs/plans/40-simulation-performance-diagnostics.md
doc_topics: [design, libraries, preparation, validation]
---

# Plan 40d: Original affine-row preparation with local dependency scope

## Purpose and ownership

This companion to [Plan 40](40-simulation-performance-diagnostics.md) develops the bounded
correction for [principal-review F06](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f06).
The outcome is preparation of selected original affine rows whose symbolic proof work follows
the coordinates that can affect those rows, while preserving the scientific meaning and
failure behavior of their original aggregated residuals. The principal review's S01/S12 and
[supporting contexts §4 and S26](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s26)
supply the motivating scenarios; their scenario identifiers have different scopes.

This file owns D0–D4 execution state. Plan 40 owns F06 disposition, shared decisions and
combined acceptance. Architecture remains with blueprint §14.1–§14.4 and §18.3; this is a
focused planning assessment using Core 3.4, Efficient Architecture Heuristics 1.0 and
ProcessSimulator 1.5, chiefly AP-02/AP-04/AP-06/AP-07, DP-09/DP-10/DP-15 and PS-06/PS-09/PS-11.

## Current source and foundation assessment

The source basis is `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` plus the existing dirty
tree inspected on 2026-10-14. Concurrent work is preserved.

`pse-math/src/class_evidence.rs::CasePlan::affine_row_facts` requires current domain facts
and the original row inventory. It selects rows lacking affine facts, aggregates their
original contributions, and leaves already established rows intact. Missing expressions,
nonlinear rows and coefficient representation limits withhold the relevant row. Cancellation,
stale assumptions and exhausted allowance remain errors. Whole-program class evidence,
original domain obligations, provenance and numerical assessment are separate responsibilities.
Those distinctions are useful foundations and must survive the correction.

`aggregate_expressions` nevertheless constructs formals and a column lookup for every plan
column, traverses bindings to find requested contributions, and rebinds each selected body
through sequential temporary-symbol substitution passes. The temporaries protect simultaneous
binding: an inserted global formal must not become a later body-local replacement target.
`prove_affine` differentiates each row against the whole formal vector and replaces each
formal with zero to recover the constant. A nonconstant derivative can stop the traversal;
an affine row traverses the full vector. Independent blocks can therefore add unrelated
coordinate work to an unchanged selected row. D0 must distinguish global indexing needed
once for the plan from repeated discovery, substitution and proof work for each demand.

`pse-compiler/src/workspace.rs::PreparedCase::prepare_affine_rows` wraps this operation and
rebuilds its problem-fact projection using the same remaining proof allowance. In
`pse-runtime/src/math/solves.rs::MathService::discover_class`, mandatory whole-program class
admission is separately conditional. Automatic presolve can still request missing affine-row
facts for an explicit Ipopt NLP selection. The fix must preserve that selected-policy demand,
including explicit presolve options and linked NLP capability, rather than avoiding the work
by changing routing or disabling presolve.

There is important contrary evidence against a broader rewrite. Compiler body admission
uses complete normalized mathematical inputs and retained immutable bodies. In
`pse-math/src/assembly.rs`, local coordinates, shared supports and grouped `LocalDemand`
requests already share derivative preparation and artifact requests across equivalent bindings.
Compiler value rebind also preserves valid plans and artifacts. This inquiry establishes
neither 32 separately compiled copies of every body nor one dense 192-axis Hessian evaluator.
Keep these owners and their sharing; affine-row proof is a distinct pass with its own demand.

The interrupted cold estimator and historical 96-variable artifact do not establish a
current cause or comparison with the declared 192-variable workload. No fraction of the
roughly 622-second estimate, regression magnitude or speedup is claimed. The warranted
correction is structural; its end-to-end quantitative benefit remains unmeasured.

## Target operation and semantic boundaries

The math owner receives the current plan, bound assumptions, original facts, selected
unresolved rows, remaining proof allowance and cancellation. It locates all contributions
needed by each selected original row, binds their body-local expressions correctly, aggregates
them using library normalization, then proves that aggregate over its actual dependent free
coordinates. It returns original-row affine facts with coefficients in the existing global
column space, preserving source attribution and fact validity. This does not require a new
crate, service, public data model or alternative production proof path.

Prove the aggregate, never each contribution in isolation. For example, contributions
`x*x + 2*x` and `-x*x + 3` yield the affine residual `2*x + 3`, even across different bodies.
A per-body rejection would lose valid evidence. Aliased slots can also cancel or combine
dependence. Scales and offsets belong in binding before aggregation, and contribution scales
belong in the row sum. Fixed and parameter values can change coefficients, the constant or
whether the aggregate is affine; they remain consumed assumptions with complete validity.

Dependency scope must be established from the bound original aggregate and exact library
semantics. A sound conservative support can guide discovery, but its false positives must
not silently become a claim of exact relevant work. Inspect whether normalized symbolic
support can supply the actual free coordinates; include nested functions and every supported
symbolic form. If completeness cannot be established, keep an honest existing refusal or
unknown outcome and resolve the dependent design in D0. Never guess missing dependencies.

Compact local proof coordinates must map explicitly back to the original global columns.
Reject unmapped or inconsistent indices. No incidental symbol-registration ordinal becomes
a column identity. Computing the constant may zero only the proven complete free-coordinate
set; a residual symbol cannot be silently treated as a numeric constant. Zero and constant
aggregates require meaningful empty support, not a bypass of domain or infeasibility checks.

Cancellation and allowance apply to support discovery, aggregation, substitution and proof
construction. Charge their actual chosen construction conservatively under the shared
remaining allowance; removing work can change the amount consumed, but must not mint a
fresh budget, suppress exhaustion or turn resource refusal into mathematical nonaffinity.
Preserve original guards, tapes, domain obligations and postsolve assessment even when
normalization cancels the guarded algebra. A proved affine residual on its original valid
domain does not establish that domain or authorize a candidate.

## Library choice and bounded investigation

Use the `symbolica-faer-oximo` skill to locate contracts, then inspect the exact installed
Symbolica 3.0.1 source and workspace features. The skill pins 3.0.0, so its API leads are not
release-qualified evidence for this checkout. Relevant supporting evidence is
[library capabilities](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/library-capabilities.md)
and the [linear supplement](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/linear-backend-diagnostics.md).
Their static evaluator counts and native timing facilities do not measure affine proof work
or settle substitution semantics. Stale capability maps or utilization entries are not a
substitute for the consumed source contract. Use Context7 resolve/query when current library
documentation is needed; record the exact release relevance of anything returned.

D0 settles two consequential choices: how complete aggregate support and original-column
mapping are obtained without repeated unrelated scans, and whether a fitting library bulk
replacement preserves simultaneous binding. Inspect traversal/rewrite behavior, replacement
ordering, inserted-expression revisiting, normalization and temporary-symbol isolation.
An API called “multiple replacement” alone proves none of these semantics. A small bounded
semantic probe during implementation is appropriate only if source leaves material doubt.

The simplest viable correction keeps the existing correct temporary substitution while
localizing coordinate proof and contribution access. Select a library bulk operation only
if its exact contract and revealing permutation/alias controls support the complete binding.
Otherwise retain the existing correct substitution: D2 records that decision rather than
requiring a blanket rewrite. A bespoke symbolic rewriter would add normalization and upgrade
obligations without an established gap and is not selected. D0 records its decision, evidence,
remaining uncertainty and dependent packets in this plan, not a separate evidence bureaucracy.

## Composition, reuse and observation

D0 can proceed independently of the diagnostic implementation. Functional correction should
consume [40a A1](40a-operation-context-and-live-history.md)'s semantic work scope and
[40b B1](40b-context-and-native-diagnostics.md)'s preparation/math projection when available.
Coordinate those shared contracts before editing overlapping producers; do not require all
diagnostics or inspection work to finish before a local math correction can proceed.

Expose bounded summaries at affine preparation entry/completion and safe checkpoints:
selected policy and demand, selected/examined/relevant original rows,
formal or dependency coordinates examined versus relevant, actual substitution/proof
construction counts, and established/withheld/refused outcomes. Distinguish indexing/setup
from per-row work and actual execution from conservative allowance units. Retained reuse
must be distinguishable from new proof work. This uses the common observation projection;
it does not emit a log for every arithmetic operation or rerun proof for inspection.

Reuse immutable row access/support only over complete structural dependencies; reuse bound
affine facts only while all consumed fixed/parameter values, bindings, contribution scales,
original obligations and relevant policy premises remain valid. Preserve the existing fact
keys and compiler/runtime validity owners unless D0 demonstrates an actual contract gap.
Free-variable iterate changes alone do not change symbolic coefficients, while fixed-value
changes can. Do not add a global proof cache to conceal repeated traversal or cache refusal
as a scientific result. Migrate actual consumers with the changed product; observers do not
reconstruct support or validity independently.

Adding independent blocks should reuse normalized bodies and valid artifacts while a selected
row's proof follows its own aggregate dependencies. Changing a contribution or fixed/free
status invalidates affected support/facts; changing a fixed coefficient invalidates affected
bound facts without gratuitous evaluator rebuilding. Upgrading Symbolica changes the math
integration and focused contract controls, without changing runtime routing policy.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
|---|---|---|---|
| D0 — Source/API decision | Current F06 source, original-row proof contract and exact Symbolica 3.0.1 source/features; independent of A packets | Settle complete support discovery, contribution access, global/local mapping and simultaneous replacement semantics. Record selected design, decisive evidence, remaining questions and which packets they constrain. A material unresolved premise blocks only its dependent change. | Done — exact-source decision and revealing controls |
| D1 — Local original-row aggregate proof | D0 support/mapping decision; coordinate A1/B1 observation slices when available | Aggregate all required contributions before proof, differentiate only actual dependent free coordinates, retain original global coefficient indices and original-domain facts. Demonstrate cross-body cancellation and independent-block work locality without dropping mandatory proof or routing demand. Move actual callers and delete the replaced global selected-row path and obsolete tests/fixtures as soon as replacement controls pass; carry any genuinely dependent migration into D3 with its deletion. | Done — targeted semantic/locality acceptance |
| D2 — Substitution correction if warranted | D0 exact replacement decision; D1 aggregation invariants | If qualified, adopt library simultaneous bulk substitution and remove replaced temporary-pass machinery and callers. Otherwise retain the correct substitution and record why. Permutations, aliases, scales, offsets and inserted-formal collision controls establish binding semantics. | Done — targeted semantic/locality acceptance |
| D3 — Reusable facts and consumer migration | D1, D2 decision and consumed A1/B1 contracts | Integrate valid structural support/access and bound facts through PreparedCase and MathService, preserving complete dependencies, remaining allowance and demand selection. Populate actual work summaries; test fixed-value invalidation, free-value reuse and no inspection-triggered proof. Complete dependent callers and delete their superseded path/interpretation/tests/fixtures in the same delivery as migration. | Done — targeted semantic/locality acceptance |
| D4 — Focused acceptance and retirement | D1–D3 integrated | Run relevant math/compiler/runtime targeted checks with explicit force-validation. Establish known mathematical outcomes, refusal boundaries and observed locality; audit immediate replacement/deletion from D1–D3 while retaining necessary whole-program class work. Hand scoped evidence and remaining qualification to 40e and F06 disposition to Plan 40. | Done — targeted semantic/locality acceptance |

## Verification and completion

**Proposed:** use independently specified small mathematical oracles, not expected values
generated by the new support transformation. Extend relevant existing controls in
`pse-math/src/assembly_tests.rs` and compiler modeling tests for original-row facts,
independent nonlinear rows, stale assumptions, allowance and cancellation; runtime strategy
controls cover Auto presolve with explicit Ipopt without requiring an expensive solve merely
to test demand policy. Planned focused commands are `just check-package pse-math`,
`just check-package pse-compiler`, `just check-package pse-runtime`, and relevant selected
`just unit-package <pkg> <filter>` invocations. Name actual filters and outcomes during execution.

| Revealing case | Required distinction |
|---|---|
| Cross-body nonlinear cancellation | The original aggregate is affine with independently known coefficients/constant, despite nonlinear individual contributions. |
| Permuted and aliased slots, binding/contribution scales and offsets | Simultaneous binding yields the known original residual and global indices; insertion cannot rebind another formal. |
| Fixed/parameter value changes and free iterate changes | Consumed coefficient assumptions invalidate affected facts; free iterate changes preserve eligible symbolic reuse. |
| Zero, constant and infeasible aggregates; guarded cancellation | Empty support is correct; original-domain obligations, infeasibility interpretation and postsolve checks survive. |
| Missing symbolic output, nonlinear row and unrepresentable coefficient | Withhold only eligible row evidence; preserve whole-program pending/ruled-out/representation distinctions. |
| Exhausted allowance, cancellation, stale facts and unmapped indices | Typed refusal/error persists, original immutable facts remain intact, no unknown becomes a false proof. |
| Two independent blocks, then an unrelated block added | Selected row outcomes remain known; actual examined coordinates and substitutions expose local work rather than a renamed global scan. Necessary one-time plan setup is identified separately. |
| Already proved rows and mixed demand | Existing facts survive, missing rows are requested by the same policy, whole-program admission remains a separate demand. |

**Proposed:** optionally inspect one current 32-block operation only after the current
observation path is usable and the source question remains decision-relevant. Preserve all
original numerical assertions and admission/solve/assessment/drain boundaries. It is not a
qualification prerequisite or comparative benchmark. No hard latency target, repeated cold
sampling, environmental equivalence or provenance-proof campaign is required.

**Proposed:** completion requires D0's decisions settled, the selected correction and all
actual consumers integrated, focused semantic/locality controls exercised and superseded
mechanisms deleted. Record actual command, mode, scope, zero-failure baseline and result;
local passes establish neither measured speedup nor assembled scientific qualification.
[Plan 40e](40e-qualification-and-closeout.md) owns end-of-scope hygiene and integration after
the combined functional scope, and Plan 40 alone closes F06. Product checks are **not_run** at authoring.

## Rule changes and checkpoint

Plan 40 owns the operator-accepted RC01 route dated 2026-10-14 for lightweight operational
observation. This companion adds no separate rule change and does not reopen that decision.
The planned mathematical locality correction is within accepted contracts; no ADR is selected
by default. If D0 exposes a changed enduring contract or new binding requiring a decision,
return that concrete choice to the coordinator before dependent implementation.

D0 source choices are settled against Symbolica 3.0.1: ordered row-to-contribution
access, reuse of the existing structural column map, encountered plan-coordinate formals,
aggregate-first candidate-symbol discovery and simultaneous `replace_multiple`. Candidate
symbols are conservative; derivative attempts and nonzero coefficients are distinct facts.
Selected-output binding uses the prepared body's existing immutable symbol-to-input map,
so a wide multi-output body does not require constructing replacements for unrelated slots.
Input symbols and mapped input slots examined are separately observed. The revealing control
compares two-slot and 64-slot bodies while demanding the same one-coordinate output.
Selected plan coordinates can include fixed variables or parameters. Request-local binding/
output reuse is allowed; no new cross-request bound-support cache is selected. Existing
facts/rebind validity, domain obligations and proof allowance remain authoritative.

The implementation removes the selected-row global scan/formal vector and temporary
substitution passes. Whole-program/objective products retain their separately required
scope. Targeted mathematical, locality, selected-coordinate and compiler rebind controls
passed D4 acceptance. Plan 40e remains a separate step.

## Outcome (recorded after implementation)

### What was built

**Implemented — 2026-10-15:** D0–D4 are complete within this packet's focused boundary.
Original-row demand uses retained row-to-contribution access, request-local binding/output
reuse and the prepared body's existing symbol-to-input map. The bound original contributions
are aggregated before conservative symbol support selects candidate free-coordinate
derivatives. Symbolica 3.0.1's simultaneous `replace_multiple` replaces temporary substitution
passes. Coefficients retain explicit original global columns; candidate derivatives and
nonzero coefficients are separate observed counts. Existing fact validity, original domain
obligations, remaining allowance, cancellation and solver demand selection remain authoritative.
Whole-program and objective products retain their required scope. The replaced selected-row
global scan/formal vector and temporary substitution mechanism are deleted.

**Tested — 2026-10-15, zero-failure baseline:** the joint targeted command below passed
**101 tests, zero failures and zero compiler warnings**, across five package binaries;
1,490 tests were not selected. It used the pinned toolchain, linked native environment,
two build jobs/test threads and the scope-aware route's explicit Arrow force-validation.
The selection also exercises 40c; it is one run, not two independent acceptance campaigns.

```bash
PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw \
CARGO_BUILD_JOBS=2 scripts/pse-env --native -- just unit-package pse-runtime \
'test(affine) | test(class_proof) | test(domain_projection_is_unassessed) | test(presolve_projection_keeps_alias) | test(inspection::) | test(observation::tests) | test(observation::target::tests) | test(capture::tests) | test(actual_private_child_publishes) | test(explicit_ipopt_auto_demands) | test(sourced_vocabularies)' \
-p pse-math -p pse-compiler -p pse-diagnostics -p pse-schema \
--features pse-runtime/native-solvers --execution-effects native-local \
--ignore-default-filter --test-threads 2
```

The controls cover cross-body cancellation, zero/constant and nonlinear rows, alias/permutation
and formal-collision binding, scales/offsets, fixed/parameter coordinate selection in reverse
order, guards/domain preservation, stale/allowance/cancellation refusal, value rebind and
retained artifact reuse, mixed policy demand and structural locality. The two-slot versus
64-slot selected-output control establishes unchanged local work counts and independently
known coefficients. These are functional/count observations, not a measured speedup or
explanation of the historical 622-second estimate. The current joint log is
`/tmp/pse40cd-joint-final.log` (Nextest `d02e0894-769f-4baa-9c43-98106b7459bc`).

### A mistake made and corrected

The first wide-body locality assertion compared every observation, including nondeterministic
phase nanoseconds. The run reported 100 passes and one failure against zero. The control now
compares the relevant `affine.*` count metrics while retaining its independent coefficient
and scope assertions. The complete affected selection then passed as recorded above.

### Deviations from the plan, deliberate

**Implemented:** no new bound-support cache, bespoke rewriter, public scientific data model
or routing shortcut was needed. Exact-source evidence and revealing controls justified the
library bulk substitution. The optional 32-block run was **not_run**: no remaining packet
decision requires it, and local count controls settle redundant traversal without a timing
campaign. Assembled qualification, scope-end hygiene/manual checks, F06 disposition and
document retirement remain with 40e and the coordinator; this packet does not close them.
