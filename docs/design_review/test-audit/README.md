# Test code design principle audit

This is the separate living diagnostic account for test code, test evidence and shared test
infrastructure. The [skill](../../../.codex/skills/design-principle-audit-tests/SKILL.md) owns
the audit method; the [repository binding](../design_principles/binding/pse-arrow.md#test-code-design-principle-audit)
owns invocation, finding identity and follow-up routes. The
[test-assessment companion](../design_principles/core/test-assessment.md) supplies the test-specific
interpretation of the selected core and applicable domain profiles.

## Baseline and coverage

- **Source baseline:** commit `cbda2efc6605db74e8a06ade00d36a72ebf97e63`, audited on
  2026-10-23. No uncommitted change is included: the working tree's dirty `history.txt` and two
  untracked plan files are unrelated to tests. The publication commit is recorded separately
  by Git.
- **Standard:** core 3.4 with its heuristics companion, test-assessment companion 1.0, the
  process-simulator profile 1.5 and the pse-arrow binding, as selected by
  [standard.toml](../design_principles/standard.toml).
- **Grounding:** source code only. Existing reviews, the broad audit's content, plans and
  capability maps were not inputs. The broad audit was consulted for finding identities and
  titles only, to keep identities distinct.
- **Architecture text:** Section 24.1 and the ADRs were treated as claims to test against the
  code; see [Claims compared with code](#claims-compared-with-code).

**Inspected scope.** These were read at the depth each question needed, with sampling, not
exhaustively:
- *Rust tests:*
  - inline and sidecar crate tests across the 28 crates, about 4,900 `#[test]`;
  - the top-level test crates (`tests/governance`, `tests/engine`, `tests/conformance`,
    `tests/lifecycle`), `tests/support` and `tests/fixtures`;
  - pse-testkit, the `fixtures`/`test-support` features of pse-quantity, pse-relations and
    pse-operations, and the proptest and insta tests.
- *Reference data:* the oracle and reference data under `packages/reference/` and its
  generator scripts.
- *Python:* the tests and the IDAES parity harness, plus the root and package `conftest.py`.
- *Execution system:*
  - `.config/nextest.toml`, pytest configuration and the justfile test recipes;
  - the Arrow validation composer and the raw and managed runner routes;
  - the resource registry and terminal reconciliation, placement, `affected` and the CI
    workflows;
  - `scripts/tests`, plus the salsa 0.28.5 and arrow 59.3.0 registry sources where decisive.

**Limits.**
- No test suite was executed, and nothing was measured. Native-only and store-only behavior
  was judged from source.
- The only probes were read-only:
  - `cargo tree`/`cargo metadata` feature queries;
  - a static check of `nextest.toml` exact-name terms ([DTA-016](evidence-routes-and-execution.md#dta-016));
  - source and history searches.
- Findings are therefore *Interface-checked* diagnoses unless stated otherwise.
- Not individually inspected:
  - most of the ~40 dynamics oracle doubles, PETSc/Uno doubles and pounce Schur tests;
  - most Python unit tests beyond targeted reads and searches;
  - the remaining conformance acceptance files (`vessel_fit`, `physical_nlp`,
    `coefficient_conic`, `connected_results_resource`);
  - `tests/lifecycle` bodies, and `native_operation.py`, `surreal_server.py` and
    `resource_store.py` internals;
  - most of the ~400 bit-exact comparisons.
- This account does not imply that the enclosing production system was audited or qualified.

## Synthesis

The suite contains strong evidence machinery, listed under [Strengths](#strengths-to-preserve).
The material weaknesses fall into four causes.

1. **Acceptance is coupled to the system it checks.** Expected values are often independent,
   but allowances come from the production numerical policy or from budgets the runtime
   publishes. Point evaluations against high-precision references accept engineering
   resolution, and a phase-regime check accepts ±10%.
   - Findings: [DTA-001](oracles-and-acceptance.md#dta-001), [DTA-002](oracles-and-acceptance.md#dta-002).
   - Consequence: accuracy regressions and route-wiring defects can widen acceptance in exactly
     the tests positioned to see them.
   - Related gaps:
     - three references lack generators ([DTA-003](oracles-and-acceptance.md#dta-003));
     - one status-mapping test is vacuous ([DTA-004](oracles-and-acceptance.md#dta-004));
     - one derivative test reuses production primitives
       ([DTA-005](oracles-and-acceptance.md#dta-005)).
2. **Some tests exercise a test-only composition or observe incidental representation.**
   - Static routing tests select through a reduced test-only path
     ([DTA-008](doubles-and-coupling.md#dta-008)).
   - Reuse tests count names in Salsa's `Debug` text ([DTA-009](doubles-and-coupling.md#dta-009)).
   - Canonical tests run a store with an extra cfg-gated lifetime field in its drop path
     ([DTA-011](doubles-and-coupling.md#dta-011)).
   - Consequence: each can pass while the production composition regresses, or fail on
     behavior-preserving changes.
3. **What ran and what passed is not always the reported outcome.**
   - The managed route exits with the child's status while its reconciled verdict can say
     failed ([DTA-012](evidence-routes-and-execution.md#dta-012)).
   - Several tests and controls have no route ([DTA-013](evidence-routes-and-execution.md#dta-013)).
   - Some qualification steps silently do nothing outside an unrecorded environment
     ([DTA-014](evidence-routes-and-execution.md#dta-014)).
4. **Guarantees differ by route.**
   - Resource association, Python grouping and the stack policy are attached to managed routes
     only ([DTA-015](evidence-routes-and-execution.md#dta-015)).
   - Exclusivity rests on unchecked exact-name lists ([DTA-016](evidence-routes-and-execution.md#dta-016)).
   - Consequence: bare and `affected` runs, which the repository presents as first-class,
     produce different and less complete evidence.

Lower-severity findings cover comparison helpers ([DTA-007](oracles-and-acceptance.md#dta-007)),
bare `is_err()` rejections ([DTA-006](oracles-and-acceptance.md#dta-006)), duplicated
cross-crate builders ([DTA-010](doubles-and-coupling.md#dta-010)), the superseded force-validate
declaration ([DTA-017](evidence-routes-and-execution.md#dta-017)), per-test fixture cost
([DTA-018](evidence-routes-and-execution.md#dta-018)) and unused shared doubles
([DTA-019](evidence-routes-and-execution.md#dta-019)).

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
| [DTA-011](doubles-and-coupling.md#dta-011) | Fixture lifetime threaded through production `CanonicalStore` | Doubles and coupling |
| [DTA-012](evidence-routes-and-execution.md#dta-012) | Managed route exit status versus reconciled verdict | Evidence routes and execution |
| [DTA-013](evidence-routes-and-execution.md#dta-013) | Committed tests and controls with no running route | Evidence routes and execution |
| [DTA-014](evidence-routes-and-execution.md#dta-014) | Env-gated qualification steps and controls return or skip silently | Evidence routes and execution |
| [DTA-015](evidence-routes-and-execution.md#dta-015) | Resource, grouping and stack guarantees only on managed routes | Evidence routes and execution |
| [DTA-016](evidence-routes-and-execution.md#dta-016) | Exclusive scheduling by unchecked exact-name filters | Evidence routes and execution |
| [DTA-017](evidence-routes-and-execution.md#dta-017) | Superseded force-validate declaration survives; name-only routes | Evidence routes and execution |
| [DTA-018](evidence-routes-and-execution.md#dta-018) | Per-test Python bridge and full schema installation | Evidence routes and execution |
| [DTA-019](evidence-routes-and-execution.md#dta-019) | Unused pse-testkit doubles; dev-dependencies beyond use | Evidence routes and execution |

## Strengths to preserve

These are working evidence patterns; a later remedy should not discard them.

**References and oracles**
- *Independent references.* Each has isolated generators that import no product code:
  - the 100-digit Decimal `log1p` oracle (`tests/fixtures/plan14/real-algebra-reference.json`,
    `crates/pse-math/src/guarded_tests.rs:397-440`);
  - the PR stability reference, validated at exact and authored constants
    (`scripts/pr_stability_reference.py`);
  - the teqp, FeOS and IDAES data generators.
- *Differential checks anchored to analytic values.*
  - `clarabel_lp_matches_highs` checks against the analytic objective and a stationarity
    residual (`crates/pse-backend-native/src/clarabel_tests.rs:184-238`).
  - The factorable projection has a relaxed-row negative control
    (`crates/pse-math/src/factorable_tests.rs:598-650`).
- *Engine oracle.* A deliberately lying provider and a known-bad over-pruning control
  (`tests/engine/tests/pushdown_vs_unpruned.rs:109,177-190`).
- *Golden vectors.* Their update rule is explicit (`crates/pse-ids/tests/golden_vectors.rs`).
- *Fixture tied to its source.* The generated standard registry is checked field by field
  against its YAML (`xtask/src/codegen/physical/tests.rs:20-28`).

**Negative cases and controls**
- *Negative cases that reach their boundary.* The domain-refusal corpus pins its inventory,
  names each refusal and has a single admitting control edit
  (`crates/pse-modeling/src/domain_schema.rs:229-270`). Relation admission uses raw literals
  and single-field boundary mutations (`crates/pse-relations/tests/admission.rs`).
- *Known-incorrect controls for shared checkers.* Conservation tests
  (`crates/pse-backend-native/src/dynamics/conservation_tests.rs`) and withheld-sensitivity
  controls for SOSC, LICQ and complementarity failures
  (`crates/pse-backend-native/src/kkt/sensitivity/tests.rs`).
- *Fault injection through real solvers.* Fatal, panic, cancel and iteration-limit outcomes,
  with quality from a fresh oracle (`tests/conformance/src/acceptance/native_outcomes.rs`).
  `FaultOracle` checks the independent post-solve validation (`crates/pse-backend-native/src/kinsol.rs`).
- *Policy seams without infrastructure* (PSE-S05). The strategy state machine is driven by
  closures (`crates/pse-runtime/src/math/strategy/tests.rs`). The default runtime fixture is
  in memory, with no store or solver.

**Execution discipline**
- *Arrow validation.* The composer derives validation from the actual unit graph and refuses
  uncovered closures (`scripts/arrow_validation.py`).
- *Python session gates.* The session refuses an unvalidated or mismatched extension. Marker
  checks run before deselection. Parity fails rather than skips, and an ast-grep rule
  enforces it.
- *Retries and replay.* Retries are 0 except in an explicit `flaky-hunt` profile with
  `flaky-result = "fail"`. Property tests have fixed seeds and committed regressions.
- *Strict Python suite.* Python product tests contain no skips and run with
  `filterwarnings = error`.
- *Evidence integrity.* Reconciled terminal evidence records `not_run` and unexpected
  identities, and fixture Drop never infers a pass.

## Claims compared with code

At this baseline, the code diverges from the architecture text and repository guidance in
several places. The divergences are recorded here as observations. The architecture is not
edited by this audit.

| Claim | Location | Code at baseline |
|---|---|---|
| Section 24.1 implies no structural test package remains | §24.1 | `tests/structural` exists locally only as untracked empty directories; Git tracks nothing there |
| "One terminal composer reconciles … missing evidence" | §24.1 | The verdict is reconciled, but the exit status ignores it ([DTA-012](evidence-routes-and-execution.md#dta-012)) |
| Composition "verifies the enabled Arrow leaf features"; bare routes carry the same obligation | §24.1; ADR-0170 | Maturin and codegen routes check the feature name only; a superseded metadata declaration remains ([DTA-017](evidence-routes-and-execution.md#dta-017)) |
| "Tests own … isolated databases and mutable runtime state" | §24.1 | True per database. Server-wide and timing exclusivity rely on exact-name lists ([DTA-016](evidence-routes-and-execution.md#dta-016)); non-managed routes leave databases pinned ([DTA-015](evidence-routes-and-execution.md#dta-015)) |
| Python tests live under `python/pse/tests` | §24.1 | Holds. Repository tooling tests are a separate unittest suite in `scripts/tests`, outside pytest `testpaths`, partly selected by text discovery ([DTA-013](evidence-routes-and-execution.md#dta-013)) |
| Bare tools are first-class | AGENTS.md | Bare pytest and cargo lose the xdist grouping, resource association and stack policy ([DTA-015](evidence-routes-and-execution.md#dta-015)) |
| `approx_eq` is the finite comparison contract | `numeric.rs`; blueprint §7.3 | It has no callers ([DTA-007](oracles-and-acceptance.md#dta-007)) |

## Reexamined and not raised

- **PSE-S05 policy tests.** The default `runtime()` fixture is local and in memory, with no store
  or solver (`crates/pse-runtime/src/workflow/env.rs:35-80`). The strategy and study policy
  seams take explicit inputs. No test-scope finding is raised. The production-structure cause
  in [DA-021](../audit/local-reasoning-and-assurance.md#da-021) keeps its own home and was not
  refreshed here.
- **The `store` nextest group.** Grouping by module path has no material scheduling cost,
  because the cap equals the test-thread count.
- **Python expectations derived through `pse`.** `test_canonical_results.py:290` (lifecycle
  re-read equality) and `test_physical_boundary.py:214` (mechanical schema conformance) are
  appropriate observations for their claims.
- **Codegen determinism.** Run-twice equality is an in-process property. Drift is checked
  separately by the `codegen … --check` recipes.
- **Insta snapshots.** They are narrow, and acceptance is by review. The round-trip property
  cannot challenge operator precedence, because both generator and renderer fully parenthesize
  (`crates/pse-authoring/tests/dsl_roundtrip.rs:10-27`). Consequence is low, given the
  authored corpus parsed elsewhere.
- **The GIL-release test.** `python/pse/tests/test_native_workflow.py:81-160` observes CPython
  3.14.8 bytecode of `Runtime.__init__`. It is deliberate, documented and singular, and the
  interpreter is pinned. It would fail spuriously on a behavior-preserving wrapper change.
- **The runtime fixture capacity floor.** `crates/pse-runtime/src/workflow/tests.rs:137-157`
  raises caller-specified capacities to at least 1 GiB of workspace. Sampled call sites do not
  depend on the requested value. The comment routes deliberate capacity tests to `runtime_on`.
- **Solver doubles.** The solver and oracle doubles inspected follow documented derivative and
  sparsity conventions and pass through production validation.

## Reading the account

Start with the baseline, synthesis and index above. The topic pages hold the findings:

- [Oracles, acceptance and assertions](oracles-and-acceptance.md): expected-result
  derivation, allowances, status mapping and negative cases.
- [Doubles, seams and contract coupling](doubles-and-coupling.md): test-only composition,
  representation coupling and production seams.
- [Evidence routes, isolation and execution](evidence-routes-and-execution.md): selection,
  gating, reconciliation, isolation and execution cost.

Findings explain what claim is at risk, why the structural cause matters, the inspected
footprint, decisive sources, contrary evidence and material uncertainty. They use the
standard's evidence vocabulary and cite applicable core and profile principles plus relevant
companion sections. Source reasoning can establish a structural weakness without running the
suite; a passing suite does not establish its evidential quality.

## Finding ownership and history

New test findings use stable DTA-nnn identities. DTA-001 to DTA-019 are assigned; the next
identity is DTA-020. Each finding has one primary home, linked from the index. Keep identity and
lineage as explanations or topic boundaries change; never reuse retired IDs.

The broad and test accounts can examine related paths at different baselines. If a structural
cause is already a finding in the broad account, it keeps its identity and home and is linked
here. Any reexamination states what was examined and any applicability limit, without copying
the diagnosis or claiming to refresh the other account. A new, distinct cause may deserve its
own finding; sharing a component or principle is not a reason to merge diagnoses. A move
records its lineage and leaves a route from the former home.

The account owns diagnosis at its stated baseline. Reviews own correction alternatives and
judgment; implementation plans own scheduled disposition, progress and closure evidence. Link
those owners without copying mutable status. Remove resolved material only on evidence,
retaining recoverable closure through the removal commit. Git holds earlier accounts; there is
no separate audit archive or execution register.

Ordinary implementation does not maintain this collection as a recurring gate. A test-scope
audit or a subsequent review uses the binding's deliberate invocation and freshness rules.
This account starts no repairs, selects no remedies and establishes no scientific or product
qualification.
