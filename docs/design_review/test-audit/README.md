# Test code design principle audit

This is the separate living diagnostic account for test code, test evidence and shared test
infrastructure. The [skill](../../../.codex/skills/design-principle-audit-tests/SKILL.md) owns
the audit method; the [repository binding](../design_principles/binding/pse-arrow.md#test-code-design-principle-audit)
owns invocation, finding identity and follow-up routes. The
[test-assessment companion](../design_principles/core/test-assessment.md) interprets the selected
standard for test scope; it does not bound the assessment.

## Baseline and coverage

- **Source baseline:** commit `cbda2efc6605db74e8a06ade00d36a72ebf97e63`, audited on
  2026-10-23 in two passes.
  - No uncommitted change is included.
  - Commits after the baseline (`14466f98e`, `c80dbfac3`, `497904dbe`) were checked. They add
    a tooling test discovered in the ordinary way, three quality-group dependencies and
    standard text. None changes test architecture or evidence.
  - Concurrent uncommitted work in pse-codegen, xtask and scripts is excluded.
  - The publication commits are recorded separately by Git.
- **Standard:**
  - core 3.4 with its heuristics companion;
  - test-assessment companion 1.1 (`497904dbe`);
  - the process-simulator profile 1.5;
  - the pse-arrow binding, as selected by [standard.toml](../design_principles/standard.toml).

  The first pass used companion 1.0. Version 1.1 makes the full standard the explicit basis and
  broadens §8 into an efficiency investigation. The second pass applies 1.1 throughout. The
  first-pass findings were rechecked under it and stand, with the corrections recorded in place.
- **Passes:**
  - *First pass:* organized by the companion's topics; produced DTA-001 to DTA-019.
  - *Second pass:* principle-directed. It reviewed the code against every core foundation, the
    operational refinements and every profile principle, and explicitly investigated execution
    efficiency. It produced DTA-020 to DTA-030 and corrected earlier findings.
- **Grounding:** source code only. Existing reviews, the broad audit's content, plans and
  capability maps were not inputs. The broad audit was consulted for finding identities and
  titles only, to keep identities distinct and to link shared causes.
- **Architecture text:** Section 24.1, ADRs and development guides were treated as claims to
  test against the code; see [Claims compared with code](#claims-compared-with-code).

**Inspected scope.** These were read at the depth each question needed, with sampling, not
exhaustively:
- *Rust tests:*
  - inline and sidecar crate tests across the 28 crates, about 4,900 `#[test]`;
  - the top-level test crates (`tests/governance`, `tests/engine`, `tests/conformance`,
    `tests/lifecycle`), `tests/support` and `tests/fixtures`;
  - pse-testkit, the `fixtures`/`test-support`/`canonical-tests` features of pse-quantity,
    pse-relations and pse-operations, and the proptest and insta tests.
- *Reference data:*
  - the authored conformance fixtures, reference data and generators under
    `packages/reference/`;
  - the modeling conformance runner.
- *Python:* the tests and the IDAES parity harness, plus the root and package `conftest.py`.
- *Execution system:*
  - `.config/nextest.toml`, pytest configuration and the justfile test recipes;
  - the Arrow validation composer and the raw and managed runner routes;
  - the resource registry and terminal reconciliation;
  - the import structure of the runner, receipt, resource and server modules;
  - placement, `affected`, `bootstrap.sh`, the CI workflows and `scripts/tests`.
- *Library and toolchain sources:*
  - the salsa 0.28.5, arrow 59.3.0 and approx 0.5.1 registry sources where decisive;
  - nextest and pytest documentation through Context7.

**Limits.**
- No test suite was executed and no new measurement was taken. Native-only and store-only
  behavior was judged from source.
- *Measured* claims come only from runs already recorded in the local nextest record store,
  with their conditions named.
- Read-only probes were:
  - `cargo tree`/`cargo metadata` feature queries;
  - a static check of `nextest.toml` exact-name terms;
  - trace exports of recorded nextest runs;
  - source and history searches.
- Other findings are *Interface-checked* diagnoses.
- Not individually inspected:
  - most of the ~40 dynamics oracle doubles, PETSc/Uno doubles and pounce Schur tests;
  - most Python unit tests beyond targeted reads and searches;
  - the remaining conformance acceptance files;
  - `tests/lifecycle` bodies;
  - most of `surreal_server.py`, `native_operation.py` and `host_admission.py`;
  - most of the ~400 bit-exact comparisons.
- Compile time per edit, xdist startup and per-test fixture cost were not measured.
- This account does not imply that the enclosing production system was audited or qualified.

## How the standard was assessed

Each passage below says how the code was reviewed against the principle and what the verdict
was. The topic pages hold the evidence.

**AP-01 Separation of concerns — finding.** Responsibility separation was reviewed in three
places: test support (pse-testkit, test-support features, `tests/support` included by path,
test modules used as helper libraries); the large sidecar files; and the runner and resource
scripts. The runner, receipt, resource and server modules form one cyclic component with several
composition roots ([DTA-026](test-execution-system.md#dta-026)). Unused shared doubles remain
([DTA-019](evidence-routes-and-execution.md#dta-019)). The 8,455-line
`workspace/modeling/tests.rs` mixes topics that have their own files, but it was not raised by
itself.

**AP-02 Stable contracts — mixed.** Tests were reviewed for dependence on incidental mechanics,
and the test-side consequences of PSE-S02 and PSE-S06 were traced:
- *What breaks on a library upgrade:* Salsa `Debug` text ([DTA-009](doubles-and-coupling.md#dta-009)),
  a few plan-text substrings and the CPython bytecode test.
- *Test-only routing composition:* [DTA-008](doubles-and-coupling.md#dta-008).
- *Structurally different production types under test:* [DTA-011](doubles-and-coupling.md#dta-011).
- *Free-text refusals:* [DTA-028](doubles-and-coupling.md#dta-028).

Typed refusals, typed engine oracles and the absence of Symbolica print assertions are strengths.

**AP-03 Composition — finding.** Scenario construction was reviewed, along with what PSE-S01
(add a model) and PSE-S03 (a new workflow) require on the test side. Authored fixtures are
discovered and checked once by shared conformance, which is a strong composition. Lower layers,
however, hand-assemble reference closures outside production resolution
([DTA-027](doubles-and-coupling.md#dta-027)). Implicit-body builders are copied across crates
([DTA-010](doubles-and-coupling.md#dta-010)). Rust test runs have several composition roots
([DTA-026](test-execution-system.md#dta-026)).

**AP-04 Domain model and semantic authority — strength with findings.**
- *Strength:* the test domain is explicit. Fixtures are authored `.pse` declarations with typed
  oracles, provenance entities and typed tolerances, and independent oracles are used widely.
- *Acceptance authority:* tolerances come from the production policy and from published budgets
  ([DTA-001](oracles-and-acceptance.md#dta-001)), in classes that do not match what each
  comparison must discriminate ([DTA-002](oracles-and-acceptance.md#dta-002)).
- *Reference authority:* three references have no generator
  ([DTA-003](oracles-and-acceptance.md#dta-003)), and the package closure has a second, manual
  authority ([DTA-027](doubles-and-coupling.md#dta-027)).

**AP-05 Explicit structure and constraints — mixed.**
- *Explicit and enforced:* Python markers, parity's failure rule, reasons on `#[ignore]`, and
  expected-failure fixtures.
- *Hidden in runner tables or compile-time features:*
  - test requirements in name-keyed tables ([DTA-023](test-execution-system.md#dta-023),
    [DTA-016](evidence-routes-and-execution.md#dta-016));
  - store-test selection as a production feature ([DTA-029](execution-efficiency.md#dta-029));
  - routeless and env-gated evidence ([DTA-013](evidence-routes-and-execution.md#dta-013),
    [DTA-014](evidence-routes-and-execution.md#dta-014));
  - an exit status that disagrees with the verdict ([DTA-012](evidence-routes-and-execution.md#dta-012)).

**AP-06 Local reasoning and independent testability — mixed.** Six responsibilities were sampled
for what a change requires a developer to read and instantiate.
- *Local:* a modeling rule, a derivative, a backend adapter and runtime strategy policy. These
  are driven by closures and an in-memory `runtime()`, so PSE-S05 holds for them.
- *Not local:* three responsibilities need unrelated infrastructure or hand-built context.
  - A canonical store operation needs a server, a Python bridge, the full schema and a feature
    variant ([DTA-018](evidence-routes-and-execution.md#dta-018),
    [DTA-029](execution-efficiency.md#dta-029)).
  - Compiler reference consumers need the hand-built closure ([DTA-027](doubles-and-coupling.md#dta-027)).
  - Pure-Python codec tests import the server supervisor through the shared conftest.

**AP-07 Execution fit and development-feedback efficiency — finding.** See
[Execution efficiency](execution-efficiency.md) for the routes followed:
- the focused Rust and Python routes, broad library units, the qualification composition and
  replay;
- in each, build, discovery and collection, setup, execution, teardown and diagnosis.

The *demonstrated* work is:
- per-test bridge and schema installation ([DTA-018](evidence-routes-and-execution.md#dta-018));
- compile-time store-test variants ([DTA-029](execution-efficiency.md#dta-029)).

The *measured* finding is that broad library feedback is bounded by a few full-closure
specializations ([DTA-030](execution-efficiency.md#dta-030)).

*Suspected and unmeasured:* the fixed pre-collection build-and-install cost of every focused
Python run, and the recompilation of each crate's whole test binary on a test edit.

**Operational refinements.** DP-01 to DP-24 were each reviewed against the test system:
- *Findings:*
  - one authority per fact, DP-01 ([DTA-022](test-execution-system.md#dta-022),
    [DTA-023](test-execution-system.md#dta-023), [DTA-017](evidence-routes-and-execution.md#dta-017));
  - library first and bespoke machinery, DP-13, DP-14 and DP-16 ([DTA-022](test-execution-system.md#dta-022),
    [DTA-007](oracles-and-acceptance.md#dta-007), [DTA-019](evidence-routes-and-execution.md#dta-019));
  - qualified library boundaries, DP-15 ([DTA-024](test-execution-system.md#dta-024));
  - structured diagnostics, DP-21 ([DTA-025](evidence-routes-and-execution.md#dta-025),
    [DTA-012](evidence-routes-and-execution.md#dta-012));
  - falsifiable claims, DP-22 (the §24.1 divergences below);
  - discoverable extension paths, DP-24 ([DTA-023](test-execution-system.md#dta-023)).
- *Strengths:*
  - enforcement points, DP-03;
  - the separation of definitions from results, DP-05: references are regenerated only by
    explicit recipes;
  - reuse keyed on complete inputs, DP-09: opt-in, digest-keyed, and failing when inputs
    change mid-run, though tool versions are missing ([DTA-024](test-execution-system.md#dta-024));
  - determinism, DP-11: single BLAS threads, no retries, fixed seeds;
  - owned workspaces, DP-19, apart from the exceptions noted.
- *Not material:* DP-07 (only `affected` uses the dependency graph, through nextest) and DP-12
  (timeouts are bounded).
- *Mixed, without findings beyond those cited:*
  - DP-02: dispositions are distinct states, but untyped strings.
  - DP-04, DP-06, DP-10, DP-17 and DP-18: no further finding.
  - DP-20: bare pytest ignores the allocation width; see
    [DTA-015](evidence-routes-and-execution.md#dta-015).
  - DP-23: the thin areas are DTA-004, DTA-013, DTA-014 and DTA-020.
- *Re-examined and not raised:* bespoke machinery with no library equivalent. That covers the
  Arrow validation composer, native provenance, cross-language terminal freezing,
  reference-protected retention, and the resource store's import fence for older worktrees.

**Process-simulator profile.** Each of PS-01 to PS-13 was reviewed for which tests carry the
obligation, whether their expectations are independent and whether a plausible defect would fail
them.
- *Strengths:*
  - PS-01: gauge and absolute, point and difference, affine temperature origin and transfer sign
    each have negative cases with admitted controls.
  - PS-02: a shared envelope probe whose rejections must name their source.
  - PS-03: known-incorrect closure controls and flowsheet-boundary accounting. The per-unit
    closure re-sums the balance's own terms, which limits its independence.
  - PS-04: exhaustive DM and BTF oracles over all small bipartite graphs.
  - PS-05: an exhaustive tear MILP enumeration with an independent DAG witness.
  - PS-06: guards that survive simplification and lowering.
  - PS-08: initialization from defaults, and failure that leaves the specification unchanged.
  - PS-10: native-success-with-failed-validation controls. The post-solve check is independent of
    the solver but shares the compiled evaluator.
  - PS-11: a steady/dynamic metamorphic fixture on one definition, and reuse observed through
    effect counters.
- *Mixed:*
  - PS-07: derivatives are first-order and start-point only in the shared check; second order is
    tested on constant-curvature problems only ([DTA-005](oracles-and-acceptance.md#dta-005),
    [DTA-020](oracles-and-acceptance.md#dta-020)).
  - PS-09: [DTA-008](doubles-and-coupling.md#dta-008).
  - PS-12: validity withholding is well controlled, but the published values are tested only on
    constant-curvature problems ([DTA-020](oracles-and-acceptance.md#dta-020)).
  - PS-13: shared conformance gives every fixture its common checks, and model families have
    references, but coverage counts definitions rather than variants
    ([DTA-021](oracles-and-acceptance.md#dta-021)) and acceptance is coupled
    ([DTA-001](oracles-and-acceptance.md#dta-001) to [DTA-003](oracles-and-acceptance.md#dta-003)).
- *Gates:* PS-G2 (structure) is the strongest area. PS-G1 and PS-G3 carry the open findings above.

**Material omissions and unresolved questions.**
- Compile and edit cost was not measured.
- No mutation or perturbation probe confirmed the suspected blind spots in DTA-002, DTA-004,
  DTA-020 and DTA-021.
- Whether the solver tests outside the `scientific` group matter was not established.
- How much of the runner component is test-specific was not established.
- Production causes behind DTA-028 and DTA-030 are reported for their test consequences only.

## Synthesis

The suite has strong evidence machinery, listed under [Strengths](#strengths-to-preserve). Its
material weaknesses fall into five causes.

1. **Acceptance is coupled to the system it checks, and coverage overstates discrimination.**
   - *Coupled allowances.* Expected values are often independent, but allowances come from the
     production policy or from budgets the runtime publishes. Point evaluations against
     high-precision references accept engineering resolution, and a phase-regime check accepts
     ±10%. See [DTA-001](oracles-and-acceptance.md#dta-001), [DTA-002](oracles-and-acceptance.md#dta-002).
   - *Second order.* Second-order outputs are tested only where curvature is constant
     ([DTA-020](oracles-and-acceptance.md#dta-020)).
   - *Coverage.* Coverage counts definitions while their default variants go unchecked
     ([DTA-021](oracles-and-acceptance.md#dta-021)).
   - *Related gaps:* [DTA-003](oracles-and-acceptance.md#dta-003) to
     [DTA-007](oracles-and-acceptance.md#dta-007) and [DTA-028](doubles-and-coupling.md#dta-028).
2. **Tests exercise compositions production does not run.** Static routing selects through a
   test-only path ([DTA-008](doubles-and-coupling.md#dta-008)). Production types gain test
   fields, probes and pauses ([DTA-011](doubles-and-coupling.md#dta-011)). Reference closures
   are hand-assembled ([DTA-027](doubles-and-coupling.md#dta-027)). Reuse is counted through
   `Debug` text ([DTA-009](doubles-and-coupling.md#dta-009)). Each can pass while the production
   composition regresses, or fail on behavior-preserving changes.
3. **What ran and what passed is not always the reported outcome.**
   - The managed route exits with the child's status ([DTA-012](evidence-routes-and-execution.md#dta-012)).
   - A profile option can make a whole selection silently `not_run`
     ([DTA-022](test-execution-system.md#dta-022)).
   - Tests and controls go without routes, or run as silent no-ops
     ([DTA-013](evidence-routes-and-execution.md#dta-013),
     [DTA-014](evidence-routes-and-execution.md#dta-014)).
   - A qualification control can drop out by rename ([DTA-023](test-execution-system.md#dta-023)).
4. **Test requirements and guarantees live in the runner, not with the tests.**
   - Requirements sit in name-keyed tables ([DTA-023](test-execution-system.md#dta-023),
     [DTA-016](evidence-routes-and-execution.md#dta-016)) and in compile-time production
     features ([DTA-029](execution-efficiency.md#dta-029)).
   - Guarantees differ by route ([DTA-015](evidence-routes-and-execution.md#dta-015)).
   - The runner re-derives tool state ([DTA-022](test-execution-system.md#dta-022)) from
     floating tools ([DTA-024](test-execution-system.md#dta-024)), inside one cyclic component
     with several roots ([DTA-026](test-execution-system.md#dta-026)).
5. **Feedback cost is dominated by work unrelated to most claims.** Per-test schema installation
   and bridge round trips ([DTA-018](evidence-routes-and-execution.md#dta-018)), and a few
   full-closure specializations that set broad library wall time
   ([DTA-030](execution-efficiency.md#dta-030), *Measured*).

Lower-severity findings are [DTA-010](doubles-and-coupling.md#dta-010),
[DTA-017](evidence-routes-and-execution.md#dta-017),
[DTA-019](evidence-routes-and-execution.md#dta-019) and
[DTA-025](evidence-routes-and-execution.md#dta-025).

## Finding index

| ID | Finding | Home |
|---|---|---|
| [DTA-001](oracles-and-acceptance.md#dta-001) | Allowances come from production policy and runtime-published budgets | Oracles and acceptance |
| [DTA-002](oracles-and-acceptance.md#dta-002) | Point evaluations at engineering resolution; phase fraction at ±10% | Oracles and acceptance |
| [DTA-003](oracles-and-acceptance.md#dta-003) | Three computed references have no committed generator | Oracles and acceptance |
| [DTA-004](oracles-and-acceptance.md#dta-004) | Status-mapping tests do not state limit meanings; HiGHS test vacuous | Oracles and acceptance |
| [DTA-005](oracles-and-acceptance.md#dta-005) | PR implicit-derivative checks reuse production primitives | Oracles and acceptance |
| [DTA-006](oracles-and-acceptance.md#dta-006) | Rejections asserted as bare `is_err()` | Oracles and acceptance |
| [DTA-007](oracles-and-acceptance.md#dta-007) | Local comparison helpers; non-finite and length gaps; finite contract unused | Oracles and acceptance |
| [DTA-008](doubles-and-coupling.md#dta-008) | Routing tests select through a test-only eligibility path | Doubles and coupling |
| [DTA-009](doubles-and-coupling.md#dta-009) | Reuse tests count Salsa `Debug` names; equality-only controls | Doubles and coupling |
| [DTA-010](doubles-and-coupling.md#dta-010) | Implicit-body test builders copied across crates | Doubles and coupling |
| [DTA-011](doubles-and-coupling.md#dta-011) | Test lifetimes, probes and pauses threaded through production types | Doubles and coupling |
| [DTA-012](evidence-routes-and-execution.md#dta-012) | Managed route exit status versus reconciled verdict | Evidence routes |
| [DTA-013](evidence-routes-and-execution.md#dta-013) | Committed tests and controls with no running route | Evidence routes |
| [DTA-014](evidence-routes-and-execution.md#dta-014) | Env-gated qualification steps and controls return or skip silently | Evidence routes |
| [DTA-015](evidence-routes-and-execution.md#dta-015) | Resource, grouping, width and stack guarantees only on managed routes | Evidence routes |
| [DTA-016](evidence-routes-and-execution.md#dta-016) | Exclusive scheduling by unchecked exact-name filters | Evidence routes |
| [DTA-017](evidence-routes-and-execution.md#dta-017) | Superseded force-validate declaration survives; name-only routes | Evidence routes |
| [DTA-018](evidence-routes-and-execution.md#dta-018) | Per-test Python bridge and full schema installation | Evidence routes |
| [DTA-019](evidence-routes-and-execution.md#dta-019) | Unused pse-testkit doubles; dev-dependencies beyond use | Evidence routes |
| [DTA-020](oracles-and-acceptance.md#dta-020) | Second-order outputs tested only at constant curvature | Oracles and acceptance |
| [DTA-021](oracles-and-acceptance.md#dta-021) | Coverage counts definitions; default variants undiscriminated | Oracles and acceptance |
| [DTA-022](test-execution-system.md#dta-022) | Runner re-derives nextest profile, grammar and identity | Execution system |
| [DTA-023](test-execution-system.md#dta-023) | Test requirements in name-keyed runner tables; dead recipe; losable control | Execution system |
| [DTA-024](test-execution-system.md#dta-024) | Floating test tools; reuse key omits tool versions | Execution system |
| [DTA-025](evidence-routes-and-execution.md#dta-025) | Fixture setup failures surface as product failures | Evidence routes |
| [DTA-026](test-execution-system.md#dta-026) | Cyclic runner/resource/server component; several composition roots | Execution system |
| [DTA-027](doubles-and-coupling.md#dta-027) | Hand-assembled reference closures outside production resolution | Doubles and coupling |
| [DTA-028](doubles-and-coupling.md#dta-028) | Free-text modeling refusals; tests match wording or any error | Doubles and coupling |
| [DTA-029](execution-efficiency.md#dta-029) | Store-test selection as a compile-time production feature | Execution efficiency |
| [DTA-030](execution-efficiency.md#dta-030) | Broad library feedback bounded by full-closure specializations (*Measured*) | Execution efficiency |

## Strengths to preserve

These are working evidence patterns; a later remedy should not discard them.

**References and oracles**
- *Independent references.* Each has isolated generators that import no product code:
  - the 100-digit Decimal `log1p` oracle (`crates/pse-math/src/guarded_tests.rs:397-440`);
  - the PR stability reference, validated at exact and authored constants;
  - the teqp, FeOS and IDAES data generators.
- *Exhaustive structural oracles.*
  - DM, BTF and matching over every bipartite graph up to 4×4
    (`crates/pse-structural/src/incidence_tests.rs:96-196`).
  - Tear MILP enumeration with an independent DAG witness
    (`crates/pse-backend-native/src/tears.rs:426-470,590-625`).
- *Differential checks anchored to analytic values.*
  - `clarabel_lp_matches_highs` (`crates/pse-backend-native/src/clarabel_tests.rs:184-238`).
  - The factorable projection, with a relaxed-row negative control.
- *Metamorphic and identity checks.*
  - A steady/dynamic same-definition fixture (`packages/reference/campaign/models/cstr-dynamics.pse`).
  - NRTL Gibbs–Duhem and homogeneity checks.
  - Caloric and PR identities at near-round-off precision.
- *Engine and fixture integrity.*
  - The engine oracle has a lying provider and a known-bad control
    (`tests/engine/tests/pushdown_vs_unpruned.rs`).
  - Golden vectors have an explicit update rule.
  - The generated standard registry is checked against its YAML.

**Negative cases and controls**
- *Negative cases that reach their boundary.*
  - The domain-refusal corpus (`crates/pse-modeling/src/domain_schema.rs:229-270`).
  - Relation admission through raw literals.
  - Gauge/absolute and point/difference refusals with admitted controls
    (`crates/pse-quantity/tests/standard_package.rs:94-217`).
- *Shared conformance checks.*
  - An envelope probe at `next_down`/`next_up`, whose rejections name their source.
  - Lineage-exact expected failures with wrong-lineage controls.
  - Every fixture gains DOF, derivative, envelope, closure and expectation rows automatically.
- *Known-incorrect controls.*
  - Conservation and closure checkers.
  - Withheld sensitivities for SOSC, LICQ and complementarity failures.
  - Native success with failed independent validation (KINSOL, Clarabel).
  - Fault injection through real solvers (`tests/conformance/src/acceptance/native_outcomes.rs`).
- *Policy seams without infrastructure* (PSE-S05).
  - The strategy state machine is driven by closures.
  - The default runtime fixture is in memory.

**Execution discipline**
- *Arrow validation.* The composer derives validation from the actual unit graph and refuses
  options it does not recognize.
- *Python strictness.*
  - The session refuses an unvalidated or mismatched extension.
  - Markers are checked before deselection, and test requirements are declared at the test.
  - pytest runs with strict config and markers, strict xfail and `filterwarnings = error`, with
    no skips in product tests.
  - Parity fails rather than skips.
- *Determinism and replay.*
  - Retries are 0 outside `flaky-hunt`.
  - Property seeds are fixed, and BLAS threads are pinned to 1 on native routes.
  - All seven `#[ignore]` uses carry reasons.
- *Evidence integrity.*
  - Reuse is opt-in and digest-keyed, and fails on mid-run input change.
  - Reconciled terminal evidence records `not_run` and unexpected identities.
  - Fixture Drop never infers a pass.
  - nextest's record store keeps attributable timing evidence.

## Claims compared with code

At this baseline the code diverges from the architecture text and repository guidance in several
places. The divergences are recorded here as observations. The architecture is not edited by this
audit.

| Claim | Location | Code at baseline |
|---|---|---|
| No empty structural test package is retained | §24.1 | `tests/structural` exists locally only as untracked empty directories; Git tracks nothing there |
| "One terminal composer reconciles … missing evidence" | §24.1 | The verdict is reconciled, but the exit status ignores it ([DTA-012](evidence-routes-and-execution.md#dta-012)), and the profile can be misread ([DTA-022](test-execution-system.md#dta-022)) |
| "Test selection belongs to nextest and pytest; no exact-name acceptance manifest exists" | §24.1 | Exact-name and name-keyed tables carry qualification gates, fixture preparation and scheduling ([DTA-023](test-execution-system.md#dta-023), [DTA-016](evidence-routes-and-execution.md#dta-016)) |
| Composition "verifies the enabled Arrow leaf features"; bare routes carry the same obligation | §24.1; ADR-0170 | Maturin and codegen routes check the feature name only; a superseded metadata declaration remains ([DTA-017](evidence-routes-and-execution.md#dta-017)) |
| "Tests own … isolated databases and mutable runtime state" | §24.1 | True per database. Exclusivity relies on name lists ([DTA-016](evidence-routes-and-execution.md#dta-016)); non-managed routes leave databases pinned ([DTA-015](evidence-routes-and-execution.md#dta-015)) |
| Python tests live under `python/pse/tests` | §24.1 | Holds. Tooling tests are a separate unittest suite in `scripts/tests`, partly selected by text discovery ([DTA-013](evidence-routes-and-execution.md#dta-013)) |
| Bare tools are first-class | AGENTS.md | Bare pytest and cargo lose grouping, width, resource association and stack policy ([DTA-015](evidence-routes-and-execution.md#dta-015)) |
| Ordinary pytest takes the allocation width | `docs/dev/agent-environment.md` | Only the runner applies it; `addopts` fixes `-n 16` ([DTA-015](evidence-routes-and-execution.md#dta-015)) |
| `approx_eq` is the finite comparison contract | `numeric.rs`; blueprint §7.3 | It has no callers ([DTA-007](oracles-and-acceptance.md#dta-007)) |

## Reexamined and not raised

- **PSE-S05 policy tests.** The default `runtime()` fixture is local and in memory
  (`crates/pse-runtime/src/workflow/env.rs:35-80`). The production-structure cause in
  [DA-021](../audit/local-reasoning-and-assurance.md#da-021) keeps its own home and was not
  refreshed here.
- **The `store` nextest group.** Grouping by module path has no material scheduling cost, because
  the cap equals the test-thread count.
- **Python expectations derived through `pse`.** `test_canonical_results.py:290` (lifecycle
  re-read equality) and `test_physical_boundary.py:214` (mechanical schema conformance) are
  appropriate observations for their claims.
- **Codegen determinism and snapshots.**
  - Run-twice equality is an in-process property; drift is checked by the `codegen … --check`
    recipes.
  - Insta snapshots are narrow and accepted by review. The round-trip property cannot challenge
    operator precedence, because both generator and renderer fully parenthesize.
- **The GIL-release test.** `python/pse/tests/test_native_workflow.py:81-160` observes CPython
  3.14.8 bytecode. It is deliberate, documented and singular.
- **The runtime fixture capacity floor.** `crates/pse-runtime/src/workflow/tests.rs:137-157`
  raises caller-specified capacities. Sampled call sites do not depend on the requested value.
- **Solver doubles.** The doubles inspected follow documented derivative and sparsity
  conventions and pass through production validation.
- **Bespoke runner machinery with no library equivalent.** The Arrow validation composer,
  native provenance, terminal freezing, reference-protected retention and the resource store's
  import fence for older worktrees.
- **Lower-severity profile observations.**
  - Integrated-fixture DOF rows are asserted from admission.
  - Scaling sanity is not a shared check; PS-13 says "where warranted", and diagnostics are
    opt-in.
  - The greedy tear heuristic is witness-validated.
  - No solved reacting fixture checks element closure; admission checks it.

## Reading the account

Start with the baseline, the assessment narrative, the synthesis and the index above. The topic
pages hold the findings:

- [Oracles, acceptance and assertions](oracles-and-acceptance.md): expected-result derivation,
  allowances, status mapping, negative cases, second-order evidence and variant coverage.
- [Doubles, seams, composition and contract coupling](doubles-and-coupling.md): test-only
  composition, production seams, reference closures and refusal representation.
- [Evidence routes, isolation and execution](evidence-routes-and-execution.md): selection,
  gating, reconciliation, isolation and per-test cost.
- [Test execution system architecture](test-execution-system.md): runner structure, requirement
  tables and tool dependence.
- [Execution efficiency](execution-efficiency.md): the feedback-route investigation and its
  findings.

Findings explain what claim is at risk, why the structural cause matters, the inspected footprint,
decisive sources, contrary evidence and material uncertainty. They use the standard's evidence
vocabulary and cite applicable core and profile principles plus relevant companion sections.
Source reasoning can establish a structural weakness without running the suite; a passing suite
does not establish its evidential quality.

## Finding ownership and history

New test findings use stable DTA-nnn identities. DTA-001 to DTA-030 are assigned; the next
identity is DTA-031. DTA-011's title was widened in the second pass with its identity kept. Each
finding has one primary home, linked from the index. Keep identity and lineage as explanations or
topic boundaries change; never reuse retired IDs.

The broad and test accounts can examine related paths at different baselines. If a structural
cause is already a finding in the broad account, it keeps its identity and home and is linked
here. Any reexamination states what was examined and any applicability limit, without copying
the diagnosis or claiming to refresh the other account. A new, distinct cause may deserve its own
finding; sharing a component or principle is not a reason to merge diagnoses. A move records its
lineage and leaves a route from the former home.

The account owns diagnosis at its stated baseline. Reviews own correction alternatives and
judgment; implementation plans own scheduled disposition, progress and closure evidence. Link
those owners without copying mutable status. Remove resolved material only on evidence, retaining
recoverable closure through the removal commit. Git holds earlier accounts; there is no separate
audit archive or execution register.

Ordinary implementation does not maintain this collection as a recurring gate. A test-scope audit
or a subsequent review uses the binding's deliberate invocation and freshness rules. This account
starts no repairs, selects no remedies and establishes no scientific or product qualification.
