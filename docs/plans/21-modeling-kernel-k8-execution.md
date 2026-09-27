---
title: Modeling kernel through K8 execution
status: done
date: 2026-09-26
parent: docs/plans/21-modeling-kernel.md
adrs: [ADR-0097, ADR-0098, ADR-0099, ADR-0100, ADR-0101]
review_sources: [docs/design_review/reviews/design_review_modeling-kernel-through-k8_2026-09-26.md]
---

# Modeling kernel through K8 execution

Authorized by the maintainer after the detailed execution review. Implement remaining
K0–K8 scope, including breaking corrections to existing contracts and the pending
K4–K7 assessment. K9's integrated campaign, R1–R3 measurements and campaign-level
knowledge-boundary audit are excluded. Targeted seed conformance and replacement tests
remain part of this work. Plan 21 owns finding dispositions; earlier packets own their
existing progress and evidence. This packet sequences the residual work, not a second
architecture authority.

## Sequence and progress

| Step | Responsibility and dependency | Acceptance and deletion | State |
|---|---|---|---|
| E0 | Reconcile scope, proposed ADRs and target review | Accurate status owners; decision/design route; no fabricated acceptance | complete; revision 59 contracts, ADR acceptance remains proposed |
| E1 | Immutable package admission, visibility and compiler reuse | Dependency/refusal and incremental/clean controls; remove repeated admission | complete; targeted checks pass |
| E2 | Typed analysis fixtures, common conformance dispatch and diagnostics; E1 | Pure/steady/initialized/dynamic/negative fixtures, lifetime and cancellation controls; delete string-only projections | complete; targeted evidence and assessment below |
| E3 | Data-owned schemes, accelerator capability references, guarded branch derivatives and caloric primitives; E1–E2 | Synthetic transformation, derivative, domain and reference controls; delete closed realization dispatch | complete; targeted evidence and assessment below |
| E4 | Prelude and sourced datasets; E1–E3 | Pure and physical checks; remove duplicate scientific literals | complete; targeted evidence and assessment below |
| E5 | Caloric methods, ideal/PR/PC-SAFT potentials; E4 | Independent oracles and thermodynamic/derivative identities | complete; targeted evidence and assessment below |
| E6 | FTPx/FPhx, equilibrium and property bindings; E5 | Named BT/saponification fixtures, initialization, envelopes and branch qualification | complete; targeted evidence and assessment below |
| E7 | Control volumes, all seed units, PID and SSLW; E6 | Shared checks and unit oracles, FD/Radau and dynamic fixtures | complete; targeted evidence and assessment below |
| E8 | Vessel, existing steady/transient fitting, results and public consumer migration; E5–E7 | Replacement behavior and ownership; delete superseded construction/workflows/tests | complete; targeted evidence and assessment below |
| E9 | Registry/generator/dependency retirement; E4–E8 replacements | All callers moved; regenerate; no production FeOS or obsolete scientific declarations | complete; targeted evidence and assessment below |
| E10 | Pending assessment and K8 conformance; E9 | Actual results against zero; separate architecture/science verdicts; K9 stays open | complete; assessment/parity recorded, 77/77 fixtures covered across retained and targeted runs |

## Binding execution decisions

- Preserve the existing checkout and unrelated changes. Authoring, pure semantics,
  compiler/Salsa, math, native algorithms and runtime keep their established owners.
- An explicit package closure supplies dependencies and physical aliases. No ambient
  package lookup, independently maintained caller merge or model-specific loader.
- Checked products cannot be mutated by consumers and retain their admitted physical
  context. The compiler's existing incremental mechanism owns reuse.
- Fixture execution is data: pure, steady, initialized, integrated or simultaneous,
  with explicit policies, samples, expected outcomes and physical/relative tolerances.
  Reuse existing operations; pure tests initialize no unrelated runtime or native solver.
- Failure classification, rule, identities, observations and locations survive every
  projection. Inconclusive, cancelled, failed and unattempted remain distinct.
- Schemes are checked data over library-owned numerical mechanisms. Accelerators are
  explicit registered capabilities. No custom solver, differentiation or factorization.
- Nested-flash derivatives are branch-scoped. Selection and admissibility are explicit;
  ties, failed alternatives, singularities and unproved crossings refuse. No claim of
  global selector smoothness or global stability follows from local evidence.
- Caloric defaults consume declared primitive functions and reference values; their
  derivatives are checked against heat capacity. Arbitrary symbolic integration is not
  presumed from the continuous-axis integral capability.
- Science lives in package data, not production Rust, Python or generated constructors.
  A needed Rust mechanism is generalized and tested synthetically in its owning packet
  before accepting the science port; Plan 21's Kernel gaps table records it.
- Preserve every original seed item. Add methane/ethane/propane and DIPPR vessel data,
  and migrate currently shipped nitrogen/Shomate and other knowledge needed to retire
  its old source schema. These are retirement obligations, not bulk Plan 20 ports.
- Production datasets, derived demonstrations and oracle inputs remain distinguishable.
  Read IDAES 2.13 for behavior only; re-express equations from their physical contracts
  and literature. No copying or mechanical translation of upstream implementations.
- Preserve current fitting capabilities while replacing their preparation path. New
  training, rolling-horizon and estimation algorithms are outside this scope.
- Delete replaced mechanisms, callers and tests together when targeted replacement tests
  pass. Preserve independent oracles, generic math tests and required live consumers.
- FeOS stays pinned in a reference-only test boundary after all production consumers
  move. Do not invent a Python binding version equivalent to the pinned Rust library.

## Verification

Use targeted `just check-package`/`just check`, `just unit-package`,
`just unit-native-package` and `just py-unit-native`, with force validation. Registry and
generator changes require `just codegen`; a pinned-family change requires
`just family-check`. Refresh the native Python extension with `just py-sync-native`.
The startup report's stale environment/native library issue must be addressed before
runtime results are claimed.

The final requested assessment is `just assessment <output-directory>`, whose existing
scope excludes performance and parity. Run authored K8 conformance and selected
`just parity-container` comparisons separately. The maintainer clarified on 2026-09-27
that corrections rerun only affected fixtures; retained passing evidence remains valid
unless a change gives a concrete reason to expect different results. Unattempted fixtures
still need evidence. A fresh complete seed rerun is not required. Do not run K9 or benchmark aggregates.
Report actual commands, modes and failures against zero; earlier receipts do not prove
changed behavior. Architecture and scientific adequacy have separate judgments.

## Current checkpoint

The K0–K8 implementation now uses one immutable package closure and checked physical
context, one generic modeling IR, library-owned mathematics and the existing native
execution/lifecycle owners. Exact dependencies, import visibility and package-scoped
quantity aliases are admitted before specialization. Checked products cannot be mutated
or rebound to a foreign physical revision. Salsa retains checked revisions and tracks
selected bindings, physical prerequisites and explicit resource limits.

The shared harness executes pure, steady, initialized, integrated and simultaneous
fixtures with structured diagnostics, original checks and physical/relative expectations.
Fixture-ID policy overrides now allow root and optimization fixtures, and different
numerical derivative perturbations, in one complete discovery run. Unknown IDs and invalid
policies refuse before execution. Every discovered fixture retains its disposition even
when detailed rows reach a cap. Cancellation, inconclusive and unattempted outcomes remain
distinct. The Rust mixed-policy, budget, lifetime and native conformance controls and
Python projections pass. All 77 seed fixtures now have passing evidence across the
retained run and selected corrections/completion checks below.

The seed is authored package data: the physical prelude and 21-element dataset; caloric
methods and sourced coefficients; ideal/PR/PC-SAFT potentials; FTPx/FPhx, SmoothVLE,
nested ideal flash, BTIdeal/BT_PR and saponification; control volumes, Feed/Product,
Heater, Mixer, Flash, all Separator selections, pressure-changer presets, heat exchanger,
CSTR and backward/Radau PFR; filtered PID with anti-windup and SSLW/currency accounting;
and the PC-SAFT/DIPPR vessel. Focused analytic, IDAES and frozen independent teqp
comparisons exercised these declarations. The expanded mixed seed has complete fixture coverage across retained and targeted
post-retirement runs, following the maintainer's selected-rerun instruction. Data provenance and exact oracle conditions live with the
packages; those comparisons do not establish empirical accuracy outside their envelopes.

Seed authoring exposed generic gaps now recorded in Plan 21's Kernel gaps table. Corrections
include preserved physical contraction in reductions, bounded shared Symbolica partials,
lexical function/child scope, interface refinement, original contribution assembly,
automatic presolve refusal of an invalid reduction, derivative-work accounting, continuous
child coordinates, original dynamic trial diagnostics and explicit body limits. Every
correction remains in its generic owner with a synthetic control. Ports additionally carry
authored incoming/outgoing maxima: ordinary specialization now enforces direction,
exclusive inputs/outputs and explicitly permitted fanout before flow-graph preparation.
Indexed port, compiler graph and native causal-recycle controls pass.

Steady, transient and mixed fitting now bind source paths through the same checked package,
retaining original checks, fixed values, bounds and per-experiment integration controls.
The focused vessel fits recover the declared 10 W input; the extra unidentifiable enthalpy
coordinate is reported through local rank limitations. Joined solve sequences preserve
warm-start policy independently of allocation reuse; rejected original checks cannot seed
a successor. Runtime/Python consumers retain repeated waits, cancellation, immutable edits,
source spans, Arrow buffer ownership and exact publication. Results share allocation leases
through the last clone/export. The refreshed extension, full native Rust suite and full
native Python suite pass, including the authored fitting and publication replacements.

Retirement is implemented. The Rust/Python builders, old composition/balance/reaction/vessel
construction, production FeOS and directional-valve providers, scientific YAML/template
relations and decoders, obsolete source operations and `pse-material` are deleted with their
consumers and obsolete tests. Physical YAML contracts, generated fit/measurement declarations
and exact source editing retain their actual owners. Reference-datum subjects now refer to
actual generic authored entities; quantity axes/type subjects refer to declared kinds.
IDAES compatibility enumerations are authored data, with an isolated upstream mapping in
the parity harness. FeOS/num-dual remain optional independent reference-test dependencies.
Generators and invariant fixtures have been regenerated and stale generated families pruned.
Post-retirement runtime/source tests pass; contract-suite failures were corrected and their
affected suites pass. Compatibility parity passes against IDAES 2.13.0. The requested
assessment attempted every check. Its early default test failures were corrected: compiler
assertions now distinguish equation outputs from exported member values, parser controls
use the declared byte limit, and source publication uses a self-contained authored fixture.
The subsequent full native Rust and Python suites pass. Static quality findings remain;
there is no clean comprehensive qualification claim.

The retained seed run passed 60 of 77 fixtures. The PR heater's derivative sample
failed because its 365 K bubble-point initial guess coincided with the fixed inlet
temperature at a 0.0001 K smoothing transition. The PR heat exchanger exceeded its
100,000-item expansion allowance, leaving 15 fixtures unattempted. The two PR fixtures
now use a 364 K inlet bubble-point initial estimate, with unchanged equations, smoothing
widths, physical specifications and oracle tolerances. Their default 1e-6 normalized
derivative step and 1e-4 tolerance pass. An explicit one-million-item allowance admits
the heat exchanger. All 17 selected fixtures pass; the earlier 60 passes were retained.
No resource refusal or unattempted fixture is counted as success.

**Retained limits.** Nested-flash derivatives are branch-local; unproved crossings refuse.
The forward vessel's sampled derivative check needs perturbation 1e-9 with relative tolerance
1e-4 near back pressure. The small one-element cubic simultaneous vessel passes with an
explicit 65,536-occurrence allowance; larger meshes can exceed the unchanged 4,096-slot
bound. The simultaneous CSTR derivative inspection needs eight million cells. Diffsol vessel
sensitivity initialization passes the exercised 1e-6/1e-8 tolerance profile; tighter attempted
controls fail. The exercised IDAS 1e-8/1e-10 profile passes. Integral sensitivities, hybrid
IDAS sensitivities, general higher-index integration and global identifiability are not
claimed. No performance campaign has run.

Architecture owners and proposed ADR scope are reconciled through the authorized
decision/design route with blueprint revision 59. Decision acceptance remains on the
ADR PR route; it is not manufactured by implementation. E10 is complete within the authorized scope. The native assessment, compatibility parity
and seed fixture evidence are recorded below against zero. Plan 21 retains the proposed
decision-PR route and excluded K9; neither is manufactured by closing this packet.

### Completed assessment

**Tested:** `just assessment build/assessment/k8-resume-20260927` attempted 38/38 checks,
with 11 unsuccessful checks against baseline zero. The checkout changed during corrective
work, which the report records; this is not an unchanged-source qualification receipt.
The default run had 20 failures among 1,424 tests. Their focused corrections pass, as do
the subsequent `just native-test --profile ci` scope (1,509 passed, zero failures/skips),
`just native-python` scope (154 passed, zero failures/skips), and `just doctest`.
The recipes supplied the native solver features and explicit force validation. Local
conditions were Linux, Rust 1.98.1 and Python 3.14.7.

**Tested:** `just parity-container python/pse/parity/tests --maxfail=0 --tb=short` passed
51 tests, zero failures/skips, against IDAES 2.13.0 under container Python 3.13.15. This
covers environment and authored compatibility names/classes/schemes, not numerical IDAES
equivalence. `just check` passes after the Rust corrections, with warnings. `just codegen`
regenerated the registry outputs.

The remaining unsuccessful static checks are Rust/Python formatting, TOML lint, both
configured Clippy modes, Python lint, typos, licensing, rustdoc and generated relation
tracking. The relation check reports new untracked generated files; regeneration itself
succeeds. No shared files were staged merely to satisfy that tracking check. Raw logs,
test findings and source drift remain in the ignored assessment directory.

### Seed completion by selected checks

**Tested:** Linux, Rust 1.98.1 and Python 3.14.7, native-solvers dev extension refreshed
by `just py-sync-native` with explicit force validation. The retained
`just modeling-conformance` mixed run had 60 passing fixtures, one failed fixture, one
inconclusive fixture and 15 unattempted fixtures. Its failures remain part of the record.
The maintainer requested targeted correction checks rather than another complete run.

| Command / selected scope | Conditions and result against baseline zero |
|---|---|
| `.venv/bin/python /tmp/k8-targeted-conformance.py ecb5f19e10f948de9aa1215714c8370e 262dd921b1bb4dd992d911cf8dcd3d92` | Native environment from `scripts/native-execution-env.sh`; PR heater and cocurrent heat exchanger: 2 passed, zero fixture failures, no diagnostic findings. Root/auto backend, presolve off, 600 s solve limit, 64 GiB pool, 1,000,000 expansion items, 1,000,000 derivative cells, step 1e-6, tolerance 1e-4. |
| `.venv/bin/python /tmp/k8-remaining-conformance.py` over the 15 previously unattempted IDs | Same native environment; 15 passed, zero fixture failures. Root/auto backend and presolve, 600 s solve limit, 128 GiB pool, 1,000,000 expansion items, explicit 65,536 body occurrences, 1,000,000 derivative cells; step 1e-6 except the forward vessel's existing 1e-9 policy; tolerance 1e-4. |

The second selection covers scalar identities, affine/indexed smoothing, four material
balance bases, the saponification pressure changer, backward/Radau PFR, closed integrated
and simultaneous vessels, forward/reverse valves and the heat meter. The native harness
executes the selected authored fixtures, including original physical and oracle checks.
The scratch selectors retain all definitions and data and remove only unselected test
declarations from admitted copies. Their whole-package coverage flag is therefore false;
it is not reported as a passing complete-package invocation. Their fixture inventories
are complete and all selected dispositions are passed. Combined with the retained run,
all **77/77 fixtures** have passing evidence and every concrete definition has fixture
coverage. The two changed starting values are fixture-local; no production mechanism
changed and no prior passing fixture was rerun.

**Implemented — architectural fitness:** the corrective ownership and retirement target
is implemented, with the synthetic/native boundary evidence recorded above. Proposed ADR
acceptance remains a separate decision process. **Tested — scientific adequacy:** the
selected seed satisfies its declared oracle, derivative, envelope and closure checks at
the recorded conditions. This establishes neither empirical accuracy beyond those
envelopes nor K9 formulation/convergence campaigns or measurements.

## Outcome

### What was built

**Implemented:** the authored seed and all live scientific replacement consumers use the
shared checked package, compiler, mathematical and native workflow owners. Production
FeOS, legacy scientific relations/builders/providers and `pse-material` are retired.
**Tested:** the 77-fixture inventory is covered by retained and selected conformance
evidence above; the completed native Rust/Python assessment and compatibility parity keep
their stated scopes. The authorized K0–K8 residual work is complete. The assessment's
unsuccessful static checks remain disclosed; no clean comprehensive qualification is claimed.

### A mistake made and corrected

**Tested:** shrinking the derivative perturbation around the PR heater's narrow smoothing
transition introduced cancellation in the earlier probe. Moving the fixture's initial
bubble estimate off that transition instead restored the default derivative check without
changing the science or relaxing tolerances. The heat exchanger required an explicit
expansion allowance; the selected run now admits it and exercises its original oracles.

### Deviations from the plan, deliberate

The maintainer replaced the final complete-seed rerun with affected-only correction checks
and retained passing evidence. The 15 unattempted fixtures were executed separately. No
K9 campaign, performance measurement, additional full assessment or formal ADR acceptance
was added. This packet remains linked by Plan 21 while it owns the completion evidence.
