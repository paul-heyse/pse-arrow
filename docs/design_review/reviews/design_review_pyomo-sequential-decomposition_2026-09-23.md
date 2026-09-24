---
title: Delegate tear selection and sequential-modular initialization to Pyomo SequentialDecomposition (ADR-0075, blueprint revision 43)
date: 2026-09-23
status: proposed
scope: ADR-0075; the blueprint revision-43 amendment (§3.1, §3.2, §3.3, §6.7, §6.13, §6.15.5, §12.5, §14.1, §17.4, §21.1, §21.2, Appendix A/B); the ADR-0063 amendment; register row R-32
depth: standard
evidence: Proposed for the route as a whole; Interface-checked for the pinned Pyomo 6.10.1 network surfaces (read, and exercised by scratch probes)
decision: Revise
---

# Design review: Pyomo SequentialDecomposition for tear selection and sequential-modular initialization

## 1. Decision and scope

**Decision: Revise.** The direction is sound. It removes two false blueprint claims and a
bespoke tear heuristic whose objective differs from the parity reference, and it hands tear
selection to maintained upstream code. But the amendment is not yet a consistent
specification:

- **It contradicts the blueprint text it leaves in place.** Examples are D12, §12.2
  `Port.Extensive` and §12.3 "no expand-arcs step".
- **It drops meaning at the Python boundary.** Duals and scaling of connection
  equations are lost, and tear non-convergence is invisible.
- **It names policies and inputs that have no declared home.** These are the stage
  options, the `fixed` tear set, tear-stream guesses and `tear_cost` validity.

Four gates fail and three are unresolved (§6). Every required change is bounded (§11).
Most are specification changes and do not reverse the decision.

**Proposal.** Move tear selection out of pass P5 and into the Pyomo adapter. Have the
adapter run `plan.sequential_modular@1` through Pyomo `SequentialDecomposition`, over unit
blocks, ports and directed arcs built from new bundle network tables and expanded by
`network.expand_arcs`. Withdraw `inferred.tear_candidates`, and record the selected tear set
per run in a new `runtime.tear_selections @1`.

**Status.** The whole route is **Proposed**: the Pyomo adapter is a 14-line docstring
(`python/pse/adapters/pyomo/__init__.py`) and `crates/pse-backend-pyomo/src/lib.rs` has 9
lines. The pinned Pyomo surfaces the ADR relies on are **Interface-checked** by this review
(§9). The P5 heuristic being withdrawn is **Implemented**
(`crates/pse-compiler/src/passes/p5/tears.rs`). Its tests exist in
`tests/engine/tests/finite_semantic_inference.rs`, but this review did not run them.

**Reviewer / author.** Claude (design-review skill) reviewing ADR-0075 (deciders:
paul-heyse).

**Affected revisions.** Blueprint revision 43 in the uncommitted working tree. The same
diff carries revisions 41 and 42 from other work; this review excludes them. Also affected:

- ADR-0075 (proposed);
- ADR-0063 (proposed; amended in place);
- `docs/adr/register.md` R-32;
- the pins Pyomo 6.10.1 (`pyproject.toml` `pyomo` extra) and IDAES 2.12.0
  (`external/idaes-pse` at tag `2.12.0`, `995ef18f`).

**Observable outcome claimed.** The tear set matches the parity reference's objective. The
tear-selection MILP, the unit solves and the final solve all go through `SolverFactory`
under named profiles. The false §12.5 claims are removed, and so is the bespoke Rust tear
code.

**Baseline (revision 40).**

- **P5 selection.** P5 selects tears with petgraph `greedy_feedback_arc_set`
  (`tears.rs:193`). It verifies acyclicity after removal (`tears.rs:222`) and rejects
  non-finite or negative costs (`tears.rs:143`). The code comment (`tears.rs:173`) and the
  earlier §6.15.5 text both disclaimed optimality. Only §12.5 claimed FOQUS reproduction
  and a minimum-FAS `mip`. The implementation was honest; the blueprint was internally
  inconsistent.
- **No native plan.** A native `plan.sequential_modular@1` was never implemented; `rg`
  finds no plan code.

**Supported scope and non-goals (as stated).** In scope are recycle initialization through
SequentialDecomposition, the four tear methods, and Arc-backed `equality` (and `extensive`)
connections. Out of scope are the other §17.2 plans, native solves, block triangularization
and the NL backend. Two items are deferred: per-unit plan lowering (R-32) and
Pyomo orchestration of other solves.

**Constraints and uncertainty.**

- **Execution-order constraint.** Plan 11 defers all component and integration workflows
  until I00–I17 close. The ADR's own verification (`sequential_modular_recycle`, a
  component test) therefore cannot run before I18.
- **Open numerical question.** No evidence yet shows that a unit-block solve from bundle
  initial values converges the Slice B flash units.

### Method and coverage

**Read in full:**

- ADR-0075.
- The revision-43 hunks of `git diff docs/authoritative_design/blueprint.md`, found by
  `grep -n ADR-0075`.
- The ADR-0063 and `register.md` diffs.
- `pyomo/network/decomposition.py` and `foqus_graph.py` (1044 and 935 lines).
- IDAES `docs/examples/structfs/hda_flowsheet.py`, the `initialize` step.
- `crates/pse-compiler/src/passes/p5/tears.rs`.

**Read at section grain:**

- blueprint D11–D13, §6.11–§6.13, §6.15.5, §12.1–§12.5, §17.1–§17.4, §21.1–§21.3, §23.2
  and §25 Slice B;
- the charter's DM and gate texts that are cited;
- `pyproject.toml` (the `pyomo` extra and the `parity` group);
- the `networkx` entries in `uv.lock`;
- `tests/governance/appendix-b-deferred.toml`;
- Plan 12's Outcome.

**Searches (rg and ast-grep).**

- `rg` for `tear_candidates|TearMethod|feedback_arc_set` and for `tear_cost`, over the
  working copy excluding `external/`, `target/` and `build/`.
- `rg` for `petgraph` in `crates/`.
- `ast-grep run -l rust -p 'greedy_feedback_arc_set($$$)' crates`, which found one match
  at `tears.rs:193`.
- `rg` for `SequentialDecomposition|select_tear_method` in `external/idaes-pse`, which
  found one file, the structfs hydrodealkylation example.

**Commands, modes and results.** The failure baseline is zero.

| Command | Mode | Result | Failures |
|---|---|---|---|
| `just doctor` | environment check, session start | exit 1: `[FAIL] env: The environment is outdated; run uv sync` (fix: `just py-sync`) | 1. Unrelated to this change; not remediated, because `uv sync` would also remove the networkx install below |
| `just adr-lint` (before writing this file) | ADR front matter, index and register lint | exit 1. `adr-frontmatter-check`: 1 problem in 75 records (review path for ADR-0075 does not exist). `adr-index-check` passed; `register-lint` passed | 1. Expected; this file resolves it |
| `just adr-lint` (after writing this file) | same | exit 0. `adr-frontmatter-check`, `adr-index-check` and `register-lint` all passed | 0 |
| `just lint-typos` | repository spell check (project-pinned `typos`) | first run: exit 2, 6 findings, all in this file (the abbreviation of the IDAES hydrodealkylation example and `…_flowsheet.py:NNN` citations). The wording was changed; the rerun exited 0 | 0 after rewording |
| `diff -q` of the skill capture against `.venv/lib/python3.14/site-packages/pyomo/network/{decomposition,foqus_graph}.py` | byte comparison | identical (installed Pyomo reports `6.10.1`) | 0 |
| `.venv/bin/python -c "…create_graph(ConcreteModel())"` before the networkx install | dev venv, CPython 3.14.7 | `imports_available False`; `DeferredImportError: The networkx module (an optional Pyomo dependency) failed to import` | finding 5 evidence |
| `uv pip install --offline --python .venv/bin/python networkx==3.6.1` | environment change authorized by the user mid-review; `pyproject.toml` and `uv.lock` untouched | installed 3.6.1 | — |
| `.venv/bin/python <scratch>/sd_probe.py` and `<scratch>/tear_probe.py` | scratch probes, not repository tests | exit 0 for both; outputs quoted in §9 | 0 |

**Not inspected.**

- Revisions 41 and 42.
- The P8/P12 connection-equation producer code.
- Any MILP solve: the `mip` and `weighted` claims rest on source reading only.
- An actual IDAES run of the hydrodealkylation flowsheet.
- No `cargo` test, `just test` or `just py-test` was run. The review changes no code, and
  the ADR's verification test does not exist yet.

**Guarantees attacked.** Five, each against source and, where marked, by execution:

- tear-set validation;
- tear-convergence failure (executed);
- exception state (executed);
- tie-breaking determinism (executed);
- the connection-equation correspondence check against the bundle's declared contents.

**Asserted and not attacked:**

- Pyomo `Port.Equality` expansion reproducing `connection.equality@1` for indexed and
  expression members.
- The Wegstein parameters.
- `SolverFactory` availability checks for MILP profiles.

## 2. Authority and lifecycle map

Cells marked *(invented)* are decisions the design has not made. That list is the evidence
for findings 2, 4 and 6.

| Concept or fact | Semantic type and identity | Authority / owner | Revision boundary | Permitted update path | Derived representations |
|---|---|---|---|---|---|
| Connection (arc) | `authored.connections` row, `connection_id` | author | model revision | change op | `inferred.topology_edges` (P5); bundle `arcs` *(derivation source unstated)* |
| Connection equations | `inferred.connection_equations @2` → `math_indexed_equations` → P12 scalar `math_equations` | P8/P12 (§12.3) | model revision | pass rerun | Pyomo `<arc>_expanded.<member>_equality` from `expand_arcs` (§21.2 step 3a). Correspondence is claimed, but the comparator is **not in the bundle** (§21.1); see finding 2 |
| Port-rule binding (`equality → Port.Equality`) | *(invented: a mapping table in §17.4 prose)* | should be `reference.connection_bindings(rule_template_id, expansion)` (§6.15.5) | registry revision | registry change | bundle `arcs.<Pyomo port rule>` |
| Unit membership of equations and variables | bundle `units` (unit → equation and variable ordinals) | *(invented: which relation, which hierarchy level, partition or overlap)* | — | — | Pyomo unit `Block`s; SD graph nodes |
| Tear cost | `authored.connections.tear_cost : f64 [n]` | author (policy) | model revision | change op | read by `weighted` only; **validity check removed with `tears.rs`** (finding 4) |
| Tear method, tear solver profile, `tear_method`, `iterLim`, `tol`, `tol_type` | "stage option" *(invented: `compiled.init_stages` has no option column, and `StageKind` has no SD kind)* | — | — | — | Pyomo `SequentialDecomposition.options` |
| Fixed tear set | *(invented: no relation)* | — | — | — | `set_tear_set` |
| Tear-stream guesses | *(invented: IDAES uses `set_guesses_for` in the hydrodealkylation example's `initialize` step)* | — | — | — | Pyomo `options["guesses"]` |
| Selected tear set | `runtime.tear_selections @1` (run, connection) | run (result) | run | ingestion | — |
| Tear convergence outcome (converged, iterations, residual) | *(invented: nothing records it)* | — | — | — | Pyomo discards it (`run()` returns `None`) |
| Unit-solve outcomes inside SD | *(invented: §17.1 requires a `runtime.runs` row per stage run)* | — | — | — | — |
| Final solve outcome | `runtime.runs` + `solutions`/`duals`/`residuals` | run | run | ingestion | duals of Arc-expanded constraints have **no equation ordinal** (finding 2) |

**Deliberately opaque behavior.** The SD inner loop is opaque to the platform plan model
(§17.1). This covers calculation order, first pass, value passing that fixes and frees
variables, and Direct/Wegstein iteration. The ADR accepts this, which is legitimate
(DM-04) if the contract names its inputs, effects, outcomes and failures. It currently names
only the tear set.

**Identity behavior.** Arc-to-connection identity is carried by the bundle `arcs` table.
Pyomo expanded constraints carry Pyomo names, not `equation_id`s. Reordering arcs in the
bundle changes the heuristic's chosen tear set among equally good sets (§9, probe T1).

## 3. Semantic contracts and invariants

| Contract or invariant | Representation | Enforcement boundary | Failure behavior | Verification evidence |
|---|---|---|---|---|
| Selected tear set breaks every cycle | `check_tear_set` (`foqus_graph.py:513`) | adapter, before use | Pyomo raises `ValueError` only on the `set_tear_set` path (`decomposition.py:974`). For `heuristic`/`mip` the adapter must call it itself. The §23.2 failure class is unstated | Interface-checked (probe T2) |
| Expanded arc constraints ↔ connection equations, one-to-one | adapter check (§21.2 step 3a) | adapter | `compile.math` | **Unenforceable as specified:** the bundle carries neither `inferred.connection_equations` nor Arc-backed equation rows (§21.1) |
| Only rules with a Pyomo port-rule binding are Arc-backed | §17.4 prose mapping | adapter / plan | `capability.backend` | Proposed. The mapping names `extensive`, which §12.2 forbids |
| Tear cost is finite and non-negative | *(was `tears.rs:143` and the row check `inferred.tear_candidates nonnegative_cost`)* | **none after the change**; the Python contract checks finite only | negative cost admitted | gap |
| Unit solve has DOF = 0 (§17.1 `solve_subset`) | — | none stated | — | gap |
| Tear streams converged within tolerance | — | none; Pyomo logs a warning and returns | indistinguishable from success | gap (probe S1) |
| Pre-flight: tear solver profile, `networkx`, `numpy` | §21.2 step 3a | adapter pre-flight | `capability.backend` | Proposed; `networkx` has no pin (finding 5) |

**Absence and uncertainty.** Five states must be distinguishable:

1. SM not run.
2. SM ran and the graph has no cycles.
3. Tears were selected and converged.
4. Tears were selected and did not converge within `iterLim`.
5. Tear selection failed (MILP not optimal, or an insufficient set).

`runtime.tear_selections` can represent only 2 and "selected", and only if it has a row per
connection. That is not specified.

**Equivalence requirements.** Tear-set equivalence is not declared per method:

- `heuristic` is deterministic given arc order;
- `mip` and `weighted` are equal-objective only, and depend on the solver, its version and
  its thread count;
- `fixed` must be identical.

Expanded-arc equivalence should be structural: the same unordered variable-ordinal pair
and the same member and index, bijectively.

## 4. Derivation and execution design

| Stage | Inputs | Output contract | Preconditions | Effects and mutable ownership | Provenance / invalidation |
|---|---|---|---|---|---|
| P5 topology | normalized connections, instances | `inferred.topology_edges` (no tear rows) | endpoints exist | none | deterministic pass |
| P8/P12 | P5, P7 | `connection_equations`, scalar equations | member match (§12.2) | none | deterministic pass |
| P15 plan generation | initializer templates, profiles | `initialization_plans`, `init_stages` | — | none | **no column carries SD options** |
| Bundle assembly (`pse-backend-pyomo`) | problem, plan | bundle + `units`/`ports`/`port_members`/`arcs` | — | none | Arc-backed connection equations **omitted** from `equations` |
| Adapter build (§21.2 steps 1–3a) | bundle | `ConcreteModel`, unit `Block`s, `Port`s, `Arc`s, `expand_arcs` | pre-flight | owns the Pyomo model | correspondence check (see §3) |
| SD run | model, options | values in the Pyomo model; tear set | guesses or non-null initials on tear destinations | fixes and frees variables in place; left fixed on exception (probe S3) | none recorded except tear rows |
| Final solve | model | solver results | SD succeeded? *(unstated)* | solver | `runtime.runs` |
| Ingestion | results | `runtime.*` | ordinals | commit | duals keyed by ordinal; expanded constraints have none |

**Relationship structures.** The design has two structures that should be one:

- `inferred.topology_edges`, the unit-level connectivity (§12.5);
- the SD graph, which `create_graph` builds from the parent blocks of Arc ports.

Nothing states or checks that they are equal (DM-34). They diverge if `units` places a port
in a block other than the instance `topology_edges` uses, for example in nested
flowsheets. Tear rows keyed by `connection_id` would then describe a different graph than
the one torn.

**Provider selection and limitations.** Pyomo's defaults are `select_tear_method="mip"`
(`decomposition.py:160`) and `tear_solver="cplex"` (`:174`). The ADR overrides both, which
is correct and would otherwise break or silently change solver selection.

**Boundary contracts.** See findings 2 and 6.

**Coherent publication.** The run's committed rows are the final solve plus tear rows. What
is committed after an SD failure is not specified.

## 5. Representative journeys

**Ordinary extension: add a new equality-like connection rule** (`connection.heat@1` and
`connection.signal@1` are equalities today). Under the ADR, any rule missing from the §17.4
prose mapping refuses the whole plan. Adding one means editing adapter-side mapping prose
(finding 1).

It should be one declaration: a `pyomo_port_rule` on `reference.connection_bindings`.
Heat and signal connections are single-member equalities, so `Port.Equality` realizes them.
Refusing them needlessly narrows the supported scope.

**Meaningful change: an author edits `tear_cost` on one connection.**

- *Good:* no recompilation happens, since tears are now a run result.
- *Not good:* under the default `heuristic` (and `mip`) the edit has no effect and no
  finding says so. A negative value is admitted, because the only check was in the deleted
  `tears.rs` (finding 4).

**Boundary: Python bridge and results.** The bundle omits Arc-backed connection equations.
Pyomo generates `ab_expanded.x_equality` (probe S2). The final solve returns duals keyed by
ordinal, and the expanded constraints have no ordinal. `runtime.duals` then silently lacks
every connection-equation dual.

With a `user_scaled` profile, step 3 attaches `scaling_factor` from equation rows. The
connection equations are no longer equation rows, so Ipopt solves them unscaled on this
route and scaled natively (finding 2).

**Interruption or failure.**

1. With `iterLim = 1` on a two-unit loop whose fixed point is `x = 2`,
   `SequentialDecomposition.run` returned `None` and raised nothing. The torn arc still
   had a residual of `0.25` (probe S1).
2. The adapter would record tear rows and proceed to the whole-model solve.
3. If that solve fails, the user sees `solve.locally_infeasible` with no evidence that
   initialization never converged.
4. If a unit function raises, the downstream unit's inlet stays fixed (probe S3,
   `B.x_in fixed: True`).
5. Any continue policy that reuses the model would then solve a changed problem
   (finding 3).

## 6. Acceptance gates

| Gate | Result | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | **Fail** | Revision 43 leaves contradicting normative text. D12 says the adapter exists "for parity testing and for Pyomo-ecosystem tools", and §21.3 lists the adapter's uses without SM. §12.2 says `Port.Extensive` "is intentionally not provided", while §17.4 maps `extensive → Port.Extensive`. §12.3 says "There is no separate 'expand arcs' step", while §21.2 step 3a adds one. The §25 phase-2 row still lists "topology closure with tear selection". The connection-equation correspondence check has no comparator in the bundle | Finding 1 (amend or delete each). Finding 2 (P8 stays authoritative; transfer the comparator) |
| G2 — Semantic fidelity | **Fail** | Connection-equation duals silently absent from `runtime.duals`. Tear non-convergence is indistinguishable from convergence | Findings 2 and 3 |
| G3 — Validity | **Unresolved** | A null initial value on a tear-destination member reaches `check_value_fix` and raises `RuntimeError("Encountered a free inlet variable …")` deep in execution (`decomposition.py:675`). `tear_cost` loses its only domain check. The stage options and the `fixed` set have no schema to validate | Finding 4 |
| G4 — Hidden behavior | **Unresolved** | The ADR says the nondeterminism "is declared", but the only declaration site is a comment on `runtime.tear_selections`. Heuristic ties depend on arc insertion order (probe T1), and no order is declared | Finding 7 |
| G5 — Consistency and recovery | **Unresolved** | No run-row layout for SD solves (§17.1 requires one per stage run). No failure class for SD failures. No workspace rule after an exception | Finding 3 |
| G6 — Transformation and reuse | **Fail** | The Pyomo SM route solves connection equations without the `equation_scales` the native route applies. No selected approximation policy covers it | Finding 2 |
| G7 — Truthful capability claims | **Fail** | `extensive` is mapped for a rule that does not exist and is forbidden. `fixed` has no authoring route. The route is unavailable in a standard `pse-arrow[pyomo]` install because `networkx` is unpinned (probe P0). §25 Slice B claims an SM plan that the specified route does not deliver (translator connection refused; unit plans not lowered) | Findings 1, 4 and 5 |

## 7. Principle findings

**What holds, and what would break without it.**

- **Correcting §12.5.** Without it, the parity claims would rest on an objective no
  implementation computes: FOQUS minimises the maximum tears per elementary cycle, then the
  total (`decomposition.py:797`; the heuristic returns the lexicographic optimum at
  `foqus_graph.py:680`).
- **Mandatory `check_tear_set`.** Without it, `select_tear_mip`'s exact `value == 1`
  readback (`decomposition.py:827`) and its unchecked `opt.solve` (`:822`) can hand SD an
  insufficient tear set.
- **Overriding the `cplex` default.** Without it, the `mip` method fails on any host without
  CPLEX.
- **Tear selection as a run result (DM-13).** Without it, a solver-profile change would have
  to invalidate compiled artifacts.
- **Explicit `capability.backend` refusal for non-port-rule connections (DM-43).** Without
  it, the route would silently drop connection equations.

Findings are in severity order.

| # | Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|---|
| 1 | Revision 43 changes the adapter's role and the connection-equation route without amending the normative text that forbids both. The blueprint now contradicts itself | DM-02, DM-43, DM-59; G1, G7 | D12 ("The adapter exists for parity testing and for Pyomo-ecosystem tools … until native equivalents exist") and D11 (native operators "own … initialization and solver workspaces") are unchanged. §21.3 is unchanged. §12.2: "Pyomo's `Port.Extensive` rule … is intentionally not provided". §12.3: "Arc expansion is a pass, not a transformation call … There is no separate 'expand arcs' step". §25 phase-2 row: "topology closure with tear selection". The ADR `blueprint:` field omits §2, §12.2, §12.3, §21.3 and §25. The `pyproject.toml` `pyomo` extra comment says "Not core: … the adapter exists for parity" | An implementer following §12.2 refuses `extensive`, while one following §17.4 builds `Port.Extensive`. `expand_arcs` then creates `splitfrac` and per-arc extensive variables that exist in no bundle table (`decomposition.py` `pass_values` requires `splitfrac` values). Their values and duals cannot be ingested, and mixing physics diverges from the platform's explicit mixer and separator templates. Following D12, the SM plan is an ecosystem-only feature; following §17.4, it is the only SM route | Amend D12 and §21.3 to name the adapter as the executor of `plan.sequential_modular@1`; this is a D-decision change under AGENTS.md, so add §2 to `blueprint:`. Amend §12.3: in the Pyomo SM route, `expand_arcs` realizes P8 equations as a verified derived form (finding 2). Delete the `extensive → Port.Extensive` mapping, or add a platform `extensive` rule by its own ADR. Correct the §25 row. Update the `pyomo` extra comment | A reviewer grep for `Port.Extensive`, `expand arcs`, "parity testing and for Pyomo" and "tear selection" across the blueprint finds no contradicting sentence. `just adr-lint` passes |
| 2 | Connection equations get a second, unverifiable authority on the Pyomo route, and their duals and scaling are lost | DM-02, DM-23, DM-42, DM-53; G1, G2, G6 | §21.1: "Connection equations of Arc-backed connections are then generated by `network.expand_arcs` … and are not also transferred as equation rows". §21.2 step 3a checks correspondence with `inferred.connection_equations`, which §21.1 does not list in the bundle. Step 3 attaches `scaling_factor` "from the bundle" per equation row. Step 4 returns duals "keyed by ordinal". Pyomo names the constraints `ab_expanded.x_equality` (probe S2) | (a) The check cannot run, so a divergence between P8 and Pyomo, such as member order, index set or orientation, goes undetected. (b) Every connection equation's dual is missing from `runtime.duals` without a finding. (c) Under `ipopt.user_scaled`, connection equations are unscaled on this route and scaled natively. The same case can converge on one backend and not the other | Keep P8/P12 as the only authority. Transfer Arc-backed connection equations as a bundle table with `equation` ordinal, `connection_id`, member, index and the two operand ordinals, marked `realized_by_arc` and not instantiated as constraints. The adapter builds the bijection expanded constraint ↔ equation ordinal, attaches scaling through it and maps duals back. A mismatch is a platform generator divergence (`internal.invariant`), not a user `compile.math` error | Targeted Python unit test on a two-unit loop: bijection complete; a dual row present for every connection `equation_id`; scaling suffix present under a user-scaled profile. Negative test: permuted member order raises `internal.invariant` |
| 3 | The sequential-modular run has no outcome contract. Pyomo reports tear non-convergence only as a log warning, and the design records nothing else | DM-08, DM-29, DM-30, DM-47; G2, G5 | `solve_tear_direct` and `solve_tear_wegstein` call `logger.warning(...)` and `return hist` at `iterLim` (`foqus_graph.py:131–132`, `:238–239`). `_run_impl` discards `hist`, and `run()` returns `None` (probe S1: residual 0.25, no exception). `run_order` frees `fixed_ins` only after a successful call (`decomposition.py:408`; probe S3). §17.1: "Every stage run is a `runtime.runs` row …; failures carry the stage". `runtime.runs.status` is a solver `TerminationStatus`. `runtime.tear_selections` has no convergence, iteration or optimality column | A non-converged recycle proceeds to the whole-model solve with tear rows present. A final-solve failure is then reported as a solver failure, and the initialization defect is invisible. After a unit exception, a reused Pyomo model carries stray fixed variables, so the problem's DOF silently changes | Define the SD outcome. After `run`, recompute the tear residual with the public `tear_diff_direct`, and record per-SCC converged status, iterations and final max error. The unit function checks solver termination and DOF = 0, and raises `plan.initialization` naming the unit. Declare run rows: one per SD stage and one per final solve, plus a child row or aggregate per unit solve. On any SD exception, discard the Pyomo model and do not solve it. Map tear-selection failure and non-convergence to `plan.initialization` with a failure policy | Unit tests: a two-unit loop with `iterLim=1` records `not_converged` and its residual; a raising unit yields `plan.initialization` with the unit id, and no final solve runs on the mutated model |
| 4 | Policies and specifications the plan reads have no declared relation, and one existing validity rule is deleted without replacement | DM-07, DM-13, DM-16, DM-43; G3 | `compiled.init_stages` (§6.11) has one `solver_profile_id`, no options column, and a `StageKind` without an SD kind. The "stage option", the `tear_solver` profile, `Direct`/`Wegstein`, `iterLim`, `tol` and `tol_type` have no column. `fixed` means "an authored tear set" with no relation. Tear guesses are absent, yet IDAES sets them on the tear destination (`external/idaes-pse/docs/examples/structfs/hda_flowsheet.py`, lines 297–313) after computing the tear set (line 294). The only `tear_cost` domain checks are `tears.rs:143` and `row_checks.rs:21` (`inferred.tear_candidates`), both deleted. `python/pse/contracts/authored.py:231` checks `finite_float` only | Two implementers invent different option encodings. A `fixed` tear set cannot be authored. Tears are chosen at run time, so an author cannot know which ports need guesses. A missing initial value on a tear-destination member raises `RuntimeError` inside SD (`decomposition.py:675`) instead of a pre-flight refusal. A negative `tear_cost` reaches the `weighted` MILP, and an authored cost is silently inert under `heuristic`/`mip` | Declare typed SD options: a stage kind or options relation, with an FK `tear_solver_profile_id` and `select_tear_method`, `tear_method`, `iterLim`, `tol` and `tol_type`. Declare the fixed tear set as authored policy, for example rows of (plan or case, `connection_id`). State that guesses are case `initial` values. Pre-flight after tear selection and before `run`: every member of every selected tear destination has a non-null initial value, or the stage declares `default_guess`. Move `finite ≥ 0` to an `authored.connections` row check. Emit an info finding when `tear_cost` is authored but the method ignores it | Registry row-check fixture (valid/violating) for `tear_cost`. Pre-flight unit test: a null-initial tear destination yields a typed refusal before any Pyomo solve |
| 5 | The supported scope and parity claims outrun the route: Slice B, networkx and "parity by construction" | DM-43, DM-59; G7 | §25 Slice B acceptance: "modular-properties initialization plan; … translator …; the sequential-modular plan converges the recycle". §17.4 refuses the whole plan for any connection without a port-rule binding, which includes `connection.translate@1` (§12.2). Unit initialization is a block solve only. IDAES calls `unit.default_initializer().initialize(unit)` and falls back to `solver.solve(unit)` only on `InitializationError` (`external/idaes-pse/docs/examples/structfs/hda_flowsheet.py`, lines 315–321). R-32's check compares with "the native unit plans", but no §17.2 template exists in `pse-plans` or `pse-runtime`. `networkx` is in `uv.lock` only as an `idaes-pse` dependency (`uv.lock:611`), and `idaes-pse` is gated to `python_full_version < '3.14'` in the `parity` group. The `pyomo` extra does not pin it (probe P0: `DeferredImportError`). The only IDAES 2.12.0 user of SD is that one docs example; `idaes/` has none | The pre-flight refuses the route in every standard `pse-arrow[pyomo]` install. Slice B acceptance cannot be met by the specified route if the translator sits on the flowsheet or a flash does not converge from a block solve. R-32's trigger is then a known prerequisite, not a contingency. "Parity by construction" holds for tear selection and the convergence loop only, not for per-unit initialization | Pin `networkx==3.6.1` (with any marker split) in the `pyomo` extra, since it is now a runtime dependency of a core route. State the supported scope: `Port.Equality`-realizable connections (bind heat and signal too), no translator on the flowsheet, and units that converge from a block solve. Either make R-32 a Slice B prerequisite, or mirror IDAES by running the unit's own plan first and the block solve only as fallback. Replace R-32's check with one that can run. Relabel the parity claim | `just py-sync` followed by the pre-flight on CPython 3.14 reports `networkx` present. A unit test shows a translator connection yields `capability.backend` naming the connection. R-32 check text reviewed against existing code |
| 6 | The network tables are a new derived representation with no stated derivation, partition rule or equality to `topology_edges` | DM-09, DM-23, DM-34, DM-42 | §21.1: `units` (unit instance → equation and variable ordinals) and `port_members` (port → variable ordinals by member). §12.1 allows expression members (IDAES `VarLikeExpression`). No text says which instance level is a unit (hierarchy, `instance_tree`), whether equations and variables partition across units, or where flowsheet-level equations live | An expression member cannot be expressed as a variable ordinal and is either dropped or mis-bound. Nested units produce Pyomo parent blocks that differ from `topology_edges` endpoints, so SD tears a different graph than the one reported. Flowsheet-level equations placed in a unit block make that unit's solve non-square | Derive `units`/`ports`/`arcs` from P5 relations with a declared rule: the leaf endpoint instance of a connection is the unit; equations are owned by at most one unit; unowned equations stay top-level. Carry expression members as expression roots or refuse them with `capability.backend`. Assert that the SD edge set equals `topology_edges` restricted to Arc-backed connections | Unit test: SD `create_graph` edges equal `topology_edges`. An expression-member fixture is either realized or refused explicitly |
| 7 | Determinism is claimed as declared but is not, and the proposed parity oracle is sensitive to arc order | DM-28, DM-40, DM-53; G4 | ADR: "That nondeterminism is declared". The blueprint has only the `runtime.tear_selections` comment. `select_tear_heuristic` returns all equally good sets, and SD takes `[0][0]` (`decomposition.py:991`). Probe T1: reversing arc insertion order flips the chosen tear from `c1` to `c2` with an equal objective (1, 1). The ADR's verification compares "the adapter's selected tear set" with IDAES, whose arc order follows Python declaration order. The `runtime.tear_selections` row shape has no key and repeats `max_cycle_tears` and `total_tears` on every row | The verification fails on ties even when both sides are optimal, or passes only by accident of order. Per-row run-level totals can disagree between rows. "No cycles" and "not run" are indistinguishable when no row is written | Declare per method: `heuristic` is deterministic given arcs ordered by `connection_id`; `mip` and `weighted` are equal-objective, with solver version and threads recorded; `fixed` is identical. Verify by objective, `check_tear_set` and converged stream values within `tol`, not by set identity (or match IDAES arc order explicitly). Give `tear_selections` a key `(run_id, connection_id)` with a row for every Arc-backed connection, and move method, profile, objective and optimality status to a per-run header | Unit test: two arc orders give equal objectives and pass `check_tear_set`. A row-contract test proves one row per Arc-backed connection |
| 8 | `mip` wraps an upstream function whose defects the design detects rather than removes, while `weighted` adds a second MILP path | DM-25, DM-43 | `select_tear_mip` calls `opt.solve` without checking termination (`decomposition.py:822`), reads `value == 1` (`:827`), and uses `1000·mct + Σx` (`:797`), which is lexicographic only while the edge count is below 1000. `check_tear_set` detects insufficiency only: a feasible but non-optimal incumbent (time limit) passes and is recorded as the method's answer | A time-limited `mip` reports a suboptimal tear set as the optimum. Two MILP builders drift | Build one platform MILP over `all_cycles(G)` with two objectives (FOQUS lexicographic, or `Σ tear_cost·x`). Check termination, round binaries with a tolerance, validate with `check_tear_set` and record the optimality status. Declare the 1000-weight limit, or use a two-stage solve | Unit tests: a solver result of 0.9999999 is read as torn; a non-optimal termination is recorded, not reported as optimal |
| 9 | Tear selection is exponential with no budget or cancellation | DM-30, DM-39 | Both methods enumerate every elementary cycle (`all_cycles`, Tarjan 1973, `foqus_graph.py:772`). The heuristic branches over cycles (`:570`). The withdrawn greedy method was polynomial. The ADR states no bound | A heat-integration network with many interlocking recycles stalls the adapter inside Python with no `runtime.resource_limit` or `runtime.cancelled` outcome | Declare a cycle-count and wall budget. Exceeding it is a typed refusal that names `fixed` as the remedy. Record the cycle count. Measure the largest target flowsheet before claiming adequacy | Unit test: a generated graph over the budget yields a typed refusal. Measurement recorded under R-32 or a new row |
| 10 | The consequences list and the evidence citations are inaccurate or not reproducible | DM-59, DM-60 | The ADR lists `s6_7_instances.rs` as declaring `inferred.tear_candidates`. It does not: that file holds `connections.tear_cost` (line 136), and the declaration is in `s6_15_semantic/realization.rs:159`. Missing from the list: `tests/conformance/fixtures/invariants/inferred-tear_candidates-unique-pk/`; registering `runtime.tear_selections` for the Appendix B coverage test (`tests/governance`); redefining the `TearMethod` enum. After deletion `petgraph` has **no** use site in `crates/` (`tears.rs:18` is the only one), yet §3.1/§3.3 still name pse-compiler's topology multigraph as a use. The cited pyomo-and-solvers and rust-graphs skills are gitignored (`.gitignore:42`), and Plan 12 is untracked. "No Rust library provides an exact minimum directed feedback arc set" is true, but the parity objective is not min-FAS, and `rustworkx_core` provides `johnson_simple_cycles` | Implementation leaves a stale fixture and an unused dependency, or fails the governance coverage test. A reader on a fresh clone cannot reproduce the Interface-checked evidence | Correct the list. Remove `petgraph` from `pse-compiler`, or label the retained uses Proposed. Cite the pinned `pyomo==6.10.1` distribution, which this review confirmed byte-identical to the capture. Narrow the driver to "no maintained Rust implementation of the FOQUS objective" | `just codegen-check`, `just governance` and `just udeps` (or `just audit-machete`) after the deletion package |

**Applicability.**

- **Group 1 (authority)** and **Group 9 (boundaries and providers)** carry the weight. The
  change is a boundary-contract and authority move.
- **Group 6 (plans, effects, failure)** applies because an initialization plan moves into
  an opaque provider.
- **Group 3 (DM-13)** covers policy versus result. **Group 2 (DM-07, DM-08, DM-09)**
  covers the new policies and absence states. **Group 10 (DM-47)** covers diagnostics of SD
  outcomes. **Group 11 (DM-53)** covers the correspondence check and the parity oracle.
  **Group 12 (DM-59)** covers claim labels.
- **Groups 7 and 8** apply narrowly. DM-34 applies to the two graphs; DM-39 and DM-40 apply
  to cycle enumeration and determinism.
- **Not applicable:** Group 4 beyond DM-16, because no template mechanism changes, and
  Group 5 beyond DM-23, because no compiler IR changes other than the withdrawal of one
  output. Revisions 41 and 42 are out of scope.

**Maturity assessment.** Omitted. The gates carry the decision, and a score would add false
precision at document stage.

**ADR-0063 amendment.** It replaces the tear sentences in Scope and Outcome consistently,
and it appends a status-history entry. That is permitted while ADR-0063 is `proposed`. The
Scope clause "Activate R-30 for instance equations, and production P4–P10 ports" reads as
a slip ("for instance equations and production …") but changes no meaning. There is no
finding.

**R-32.** The row is well-formed and its trigger is observable. The deferral is not fully
honest (finding 5): §25 already requires the capability R-32 defers, and its check compares
against native unit plans that do not exist.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risks | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| **Baseline (rev 40):** petgraph greedy in P5; native SM plan | One connection-equation authority. Tear objective differs from the reference | Wrong objective for parity. Native SM never built | Low (tears.rs exists); native SM unbuilt | none | Correctly rejected on objective |
| **Proposed (ADR-0075):** SD in the adapter executes the whole plan | Adds a second connection-equation realization, network tables and SD options outside the plan model | Findings 1–9. Unit plans simplified (R-32). Optional-extra dependency | Adapter (unbuilt) plus network tables plus correspondence and dual mapping | none | Maintainer direction: Pyomo orchestrates solvers, and SD gives procedure parity |
| **Simpler viable: Pyomo selects tears; the platform plan model runs SM** | P8 remains the only connection-equation authority. No network tables. Tear rows keyed by `connection_id` from `topology_edges` | Tear parity by construction: the same FOQUS code and objective. Probe T1 shows `select_tear_heuristic` and `check_tear_set` work on a plain `MultiDiGraph` built from edge rows, with no Blocks, Ports, Arcs or `expand_arcs`. §17.2 unit plans keep their `estimate` stages, so there is no R-32 regression. Duals and scaling are unchanged | A graph build from `topology_edges`; a Direct/Wegstein loop (~60 lines, the procedure §17.4 already specifies); `propagate` and `solve_subset` from §17.1. Unit solves still go through `authored.solver_profiles`, whose `backend` enum already includes `pyomo` and the NL solvers | none | Not in the ADR's Options. Option 2 (port FOQUS to Rust) is a different alternative |

**What the comparison shows.** The ADR's "swappable solvers" driver does not discriminate
between the proposed design and the third row. Solver swapping by profile already exists
through `authored.solver_profiles.backend`. Parity of tear *selection* is equal in both.

The proposal's distinctive gain is procedure parity: Pyomo's `pass_values` and the
convergence loop. It pays for that with the boundary change behind findings 1, 2 and 6 and
the R-32 simplification.

This review does not choose for the author. The ADR should present the third row in its
Options with the maintainer's reason for preferring SD execution, or narrow its scope (§11).

**Abstractions justified by current needs.** A per-run tear-selection result is justified.
Of the network tables, the only essential part is the arc list: `create_graph` needs Arcs,
but tear selection alone does not.

**What remains ordinary code.** The `weighted` MILP and the unit function are ordinary
adapter code behind the SD stage contract (finding 8). No DSL is warranted.

## 9. Verification and measurement plan

**Scratch probes run by this review.** They execute in the dev `.venv` (CPython 3.14.7,
Pyomo 6.10.1, networkx 3.6.1 installed with user authorization). They are **not
repository tests**; each result is Interface-checked by execution.

- **P0** (before the networkx install): `create_graph` →
  `DeferredImportError: The networkx module (an optional Pyomo dependency) failed to import`.
- **S1:** two-unit loop, Direct, `iterLim=1`, `tol=1e-10`, fixed point `x=2`. `run()`
  returned `None`. Values were `A.x_in 1.5`, `A.x_out 1.75`, `B.x_in 1.5`; the torn arc
  `ab` had residual `0.25`; no exception was raised.
- **S2:** the expanded constraints are `ab_expanded.x_equality` and `ba_expanded.x_equality`.
- **S3:** unit B raises → `B.x_in fixed: True` after the exception.
- **S4:** the defaults are `mip` and `cplex`.
- **T1:** parallel edges `c1: A→B` and `c2: B→A`. Insertion order `[c1, c2]` chooses
  `['c1']`; order `[c2, c1]` chooses `['c2']`. Both give max/total `1 1`, and both sets are
  listed as optimal.
- **T2:** `check_tear_set(G, [])` → `False`; with the shared edge → `True`.

| Claim or risk | Evidence label | Test / analysis | Conditions and expected result | Current result or gap |
|---|---|---|---|---|
| Pyomo SD surfaces cited by the ADR | Interface-checked | Source read, byte identity with the installed 6.10.1, probes S2–S4, T1–T2 | pinned 6.10.1 | ADR claims confirmed: directed and expanded Arcs required (`decomposition.py:735–746`), objective (`:797`), readback (`:827`), `cplex` default (`:174`), `set_tear_set` path validated (`:974`) |
| Heuristic is an exact branch-and-bound for the lexicographic objective | Interface-checked (source) | `foqus_graph.py:525–680`: pruning is monotone in both criteria | — | holds by reading; not tested on large graphs |
| Tear non-convergence is recorded | Proposed → gap | unit test (finding 3) | `iterLim=1` loop ⇒ `not_converged`, residual recorded | Pyomo is silent (S1); design records nothing |
| Connection-equation correspondence and dual mapping | Proposed → gap | unit test (finding 2) | bijection; duals for all connection `equation_id`s | comparator absent from the bundle |
| `tear_cost` domain | gap | registry row-check fixture | negative is rejected | check deleted with `tears.rs` |
| Tear-destination guesses | gap | pre-flight unit test | null initial ⇒ typed refusal before solve | Pyomo raises mid-run (`:675`) |
| Parity with IDAES (hydrodealkylation-like recycle) | Proposed | component `sequential_modular_recycle` (ADR front matter), deferred to I18 by Plan 11 | objective equality plus converged streams within `tol`, not set identity (T1) | test not written |
| Route availability in a `pse-arrow[pyomo]` install | gap | `just py-sync` then pre-flight | `networkx` present | absent on 3.14 (P0; lock marker) |
| Cycle-enumeration cost | hypothesis | measurement on the largest target flowsheet | within declared budget | none |

**Cost accounting.** The material new costs are:

- bundle network tables and the correspondence map (one pass over connection equations);
- exponential cycle enumeration (finding 9);
- `units × iterations` unit solves through Python.

None is measured. No performance claim is made by the ADR, and none should be until one is.

## 10. Exceptions and unresolved decisions

**Suggested exception record: DM-27 (SHOULD).**

- **Principle IDs:** DM-27.
- **Scope:** the inner workflow of `plan.sequential_modular@1` is opaque Pyomo behavior,
  not `init_stages`. It covers calculation order, first pass, value passing and tear
  iteration.
- **Reason:** procedure parity with Pyomo and IDAES, and the maintainer's direction.
- **Alternatives:** the §8 third row.
- **Consequence:** the plan cannot be explained stage by stage from relations.
- **Compensating controls:** record the calculation order, tear set, per-iteration residual
  history (`hist`) and per-unit outcomes (findings 3 and 7).
- **Evidence:** this review.
- **Owner:** paul-heyse.
- **Revisit trigger:** R-32 fires, or a second consumer needs stage-level inspection.

**MUST-level items: no exception possible.** DM-02, DM-07, DM-08, DM-30, DM-42 and DM-43
(findings 1–6) must be resolved or the scope narrowed.

**Decisions the author must make:**

1. SD executes the plan (the proposal) or only selects tears (§8 row 3).
2. Whether `extensive` exists at all.
3. Whether Slice B depends on R-32.
4. Where stage options and fixed tear sets live.

## 11. Decision and implementation changes

**Decision: Revise.**

**Reason.** G1, G2, G6 and G7 fail on text in the amendment itself; G3, G4 and G5 are
unresolved. The ADR's Pyomo evidence is accurate, and the withdrawal of the P5 heuristic is
justified.

**Narrower acceptable scope.** An **Accept-scoped** outcome is available if the ADR narrows
to four items and defers SD execution of the plan and the bundle network tables to a
follow-up ADR:

1. Withdraw P5 tear selection and `inferred.tear_candidates`.
2. Correct §12.5.
3. Select tears through Pyomo's FOQUS code over `inferred.topology_edges`, recorded in a
   keyed `runtime.tear_selections`.
4. Move the `tear_cost` check.

That package has no consumer that breaks: no native SM exists.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 | Amend D12 and §21.3 (and the D11 wording), §12.3 and §25. Delete `extensive → Port.Extensive`. Add §2, §12.2, §12.3, §21.3 and §25 to `blueprint:` | DM-02, DM-43, DM-59 | No contradicting sentence remains (finding 1 grep). `just adr-lint` passes | review of any later `Port.Extensive` mention |
| 1 | P8 stays authoritative. Transfer Arc-backed connection equations as a non-instantiated comparator table. Use the bijection for scaling and duals. A mismatch is `internal.invariant` | DM-02, DM-23, DM-42, DM-53 | unit test (finding 2) | the same test |
| 1 | Define the SD outcome contract: convergence status and residual, unit-solve checks, run rows, discard the model on exception, `plan.initialization` mapping | DM-08, DM-29, DM-30, DM-47 | unit tests (finding 3) | the same tests |
| 1 | Declare SD stage options, the fixed-tear-set relation and the guess source. Add a pre-flight for tear-destination initial values. Add an `authored.connections.tear_cost` row check. Emit an info finding for inert `tear_cost` | DM-07, DM-13, DM-16, DM-43 | row-check fixture; pre-flight test | generated invariant cases |
| 1 | Pin `networkx` in the `pyomo` extra. State the supported scope: connections realizable by `Port.Equality`, including heat and signal through `reference.connection_bindings`, with translator refusal named. Make R-32 a Slice B prerequisite, or run the unit plan with block-solve fallback. Fix R-32's check. Relabel "parity by construction" | DM-43, DM-59 | pre-flight on 3.14 after `just py-sync`; refusal test | `just family-check` does not cover Python; add a pre-flight unit test |
| 2 | Derive the network tables from P5 with a unit, partition and hierarchy rule. Handle expression members. Assert SD edges equal `topology_edges` | DM-09, DM-23, DM-34, DM-42 | unit test (finding 6) | the same test |
| 2 | Declare the determinism class per method. Key `tear_selections` with a per-run header. Change the parity oracle to objective plus streams | DM-28, DM-40, DM-53 | unit test (finding 7) | the same test |
| 2 | One platform MILP for `mip` and `weighted`, with termination check and tolerant readback | DM-25, DM-43 | unit tests (finding 8) | the same tests |
| 2 | Add §8 row 3 to the ADR's Options with the reason for choosing SD execution | DM-59 | ADR text | design review of the revision |
| 3 | Declare a cycle budget with a typed refusal. Measure before claiming adequacy | DM-30, DM-39 | budget test; recorded measurement | register row |
| 3 | Correct the consequence list. Remove the unused `petgraph` dependency. Cite reproducible evidence | DM-59, DM-60 | `just codegen-check`, `just governance`, `just udeps` (or `just audit-machete`) with zero failures | the same recipes in CI |

**Final check.** The ADR's claims about the pinned Pyomo surfaces match the evidence. Its
supported scope does not yet match what the specified route can deliver. Later extensions
(new connection rules, new tear methods) do not yet have a single declared place to land.
The revision above provides one.

## 12. Re-review of the narrowed revision (2026-09-23)

**Target.**

- ADR-0075, retitled "Select flowsheet tears through Pyomo at run time and withdraw
  compiler tear selection". Its `blueprint:` list is §3.1, §3.2, §3.3, §6.7, §6.13,
  §6.15.5, §12.5, §14.1, §17.4, §21.3 and §25.
- The narrowed revision-43 text of the blueprint.
- The ADR-0063 sentence.
- Register row R-32.

Revisions 41 and 42 remain out of scope.

**Verdict: Revise, minor.** The narrowed package is the §11 Accept-scoped package, and it
resolves every gate failure of §6 except one new reuse-key defect (R1 below). Two
specification changes are needed before `status: accepted`:

- **R1:** bind the selection consumed by the plan to its full policy.
- **R2:** declare where the plan's tear policy lives.

Each is a sentence or a column. When both land, the verdict becomes **Accept-scoped**
without another full review. R3–R6 are non-blocking, and may be settled when the code is
written.

### 12.1 Contradiction check

Each grep ran on `docs/authoritative_design/blueprint.md` in the working tree.

| Check | Command | Result | Assessment |
|---|---|---|---|
| `Port.Extensive` mapping removed | `grep -n "Port.Extensive"` | 1 hit: §12.2 ("intentionally not provided") | No contradiction; the §17.4 mapping is gone |
| SD execution removed | `grep -n "SequentialDecomposition"` | 3 hits: revision row; §17.4 "deferred to a follow-up ADR (register R-32)"; Appendix A row | No normative SD execution remains |
| Arc expansion (§12.3) | `grep -n "expand_arcs\|expand arcs"` | §12.2 rule table, §12.3 ("no separate 'expand arcs' step"), Appendix A | §12.3 is no longer contradicted: §21.2 has no step 3a |
| Bundle network tables and step 3a | `grep -n -E "network tables\|^3a\.\|Arc-backed"` | 0 hits | Removed |
| §21.1/§21.2 restored | `git diff -U0 <blueprint> \| awk '/^@@/'` | No hunk between the §21.3 hunk (`+4203,13`) and the §23.2 hunk (`+4330,7`); §21.1 and §21.2 (about lines 4150–4200) have no hunk | Both are identical to HEAD, including their `> Decision` markers |
| D12 adapter role | `grep -n "exists for parity testing\|Uses of the adapter"` | D12 unchanged; §21.3 now lists "tear selection" beside parity testing and the Pyomo ecosystem | Consistent. `select_tears` is a Pyomo-ecosystem use (`pyomo.network`'s FOQUS code and `SolverFactory`). It reads a snapshot relation, not a `CanonicalMathProblem` bundle, which D12 does not forbid |
| Withdrawn output | `grep -c "tear_candidates"` | 1 hit (the revision row) | §6.7, §6.15.5, §14.1 (P5 row) and Appendix B no longer list it |
| Removed Rust claim | `grep -n "greedy_feedback_arc_set"` | 0 hits | §3.1/§3.3 now say tear selection is `select_tears`, not petgraph |
| §25 | `grep -n "tear selection"` | The phase-2 row reads "topology closure; tear selection through the Pyomo adapter (ADR-0075)" | Stale text corrected |
| New failure code in §23.2 | `awk '/^### 23.2/,/^## 24/' \| grep -c tear_cycle_budget` | 0 | See R4 |
| Tear policy in the plan relations (§6.11) | `awk '/^### 6.11/,/^### 6.12/' \| grep -ci tear` | 0 | See R2 |

The ADR-0063 sentence now reads "a run-time result of the Pyomo adapter's `select_tears`
(ADR-0075)". It is consistent, and the ADR is still `proposed`, so the in-place edit is
permitted.

R-32 now defers exactly the SD-execution findings of §7 (findings 2, 3 and 6, and part of 4
and 5). Its check can be run: does a follow-up ADR superseding ADR-0015 exist, and is the
native plan implemented? That repairs the §7 finding 5 objection to R-32.

### 12.2 Gates for the scoped package

| Gate | Result | Evidence |
|---|---|---|
| G1 — Authority | **Pass** | No contradicting normative text (§12.1). The connection-equation authority is unchanged (P8 only). The unit graph is the one P5 persists, so there is no second graph (§7 finding 6 does not arise) |
| G2 — Semantic fidelity | **Pass** | `runtime.tear_selections` has one row per topology edge, so "no cycles" differs from "not run". The run header carries method, profile, cycle count, objective terms and determinism. See R4 on non-optimal termination |
| G3 — Validity | **Pass**, with R2 | `tear_cost` is finite and ≥ 0 under a row check (§6.7). Acyclicity is validated before any row is written. Pre-flight covers `networkx`, `numpy` and the MILP profile. The cycle budget is a typed refusal. The `fixed` method is dropped rather than left unauthorable |
| G4 — Hidden behavior | **Pass** | A per-method determinism class is declared (`edge_order`, `solver_dependent`) and the semantic-ID edge order is stated (§21.3). Pyomo's `cplex` default is excluded |
| G5 — Consistency and recovery | **Pass** | Rows are written only after validation. Budget, pre-flight and termination failures refuse. SD execution, the source of the §7 finding 3 defects, is out of scope |
| G6 — Transformation and reuse | **Fail (narrow)** | §17.4 matches a selection run to the plan by `graph_hash` alone, "for the current `inferred.topology_edges`". See R1 |
| G7 — Truthful capability claims | **Pass** | Scope and claims now match: parity is claimed for tear *selection* only. The missing `networkx` pin is stated, with pre-flight refusal until it lands. SD execution is explicitly deferred |

### 12.3 Remaining changes

| # | Change | Principle IDs | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| R1 (blocking) | The reuse key that binds a selection run to the plan is incomplete | DM-31, DM-32; G6 | §17.4: "a plan without a successful selection run for the current `inferred.topology_edges` (`graph_hash`) fails its precheck". `graph_hash` is not defined as including `tear_cost`, and the match ignores method and MILP profile. Several matching runs may exist | (a) The author edits `tear_cost` without changing topology, so `graph_hash` still matches and the plan uses tears chosen for the old costs. (b) The plan's policy switches from `heuristic` to `mip`, and an earlier heuristic run still matches. (c) Two matching runs leave the choice unspecified. In each case the tear set is still acyclic, but the declared policy is silently not applied | Match on the exact tuple: `graph_hash` (the §5.3 canonical hash of `topology_edges`, plus `tear_cost` for `weighted`), method and `tear_solver_profile_id`. Alternatively, have the plan stage bind an explicit `tear_selection_run_id`. Define `graph_hash` as a canonical hash | Unit test: a `tear_cost` change or a method change makes the precheck fail. Two identical-key runs resolve deterministically, or are rejected |
| R2 (blocking) | "The plan's tear policy" (method and MILP profile) has no declared relation | DM-13, DM-16 | §17.4 and §21.3 refer to it. `compiled.init_stages` (§6.11) has one `solver_profile_id` and no tear column (grep: 0 hits). The caller that invokes `select_tears` before the native plan is also unstated | Implementers invent different encodings, and R1's key has nothing declared to compare against | Declare the policy: for example `tear_method` and `tear_solver_profile_id [n]` on the plan or case policy, with an FK to `authored.solver_profiles`. State who runs `select_tears`: the plan runner's precheck, or an explicit API | Registry fixture for the new columns. Precheck test using the declared policy |
| R3 | The execution identity of `tear_selection_runs.run_id` is undefined | DM-12 | `runtime.runs.solver_profile_id` is non-null (§6.13), but `heuristic` runs have no profile. Nothing says whether `run_id` is an FK to `runtime.runs` | A heuristic selection cannot be a valid `runtime.runs` row, and an independent `run_id` has no case or revision linkage | State the relation: an FK to a `runtime.runs` row with a nullable profile for non-solver operations, or an independent operation identity carrying `model_revision_id`/`case_id` | Registry FK check |
| R4 | Failure details are incomplete | DM-08, DM-47 | `plan.tear_cycle_budget` is absent from §23.2 (0 hits), and the budget value has no declaration site. "Termination is checked" does not say whether a feasible but non-optimal MILP result is refused or recorded. `1000·max_cycle_tears + Σx` is lexicographic only below 1000 edges | A time-limited MILP result may be reported as optimal. The budget cannot be audited | Map `plan.tear_cycle_budget` to a §23.2 class (`runtime.resource_limit` or `plan.initialization`). Declare the budget (engine setting or policy). Refuse non-optimal termination, or record an optimality status on the run header. Declare the 1000-edge limit, or use a two-stage solve | Unit tests named in the ADR `verification:`, plus a non-optimal-termination case |
| R5 | Acyclic flowsheets need Python for no reason | DM-43, DM-58 | §17.4 fails the precheck without a selection run, even when `topology_edges` is acyclic | A native-only install (no `pyomo` extra) cannot run `plan.sequential_modular@1` on a flowsheet with no recycle | Exempt acyclic graphs: no tears are needed, and a native SCC/acyclicity check suffices. Or state the dependency as intended | Unit test: an acyclic topology passes the precheck without a selection run |
| R6 | Evidence and deletion hygiene | DM-59, DM-60 | The ADR Confirmation still cites the gitignored pyomo-and-solvers skill and this review's scratch probe (under the session scratchpad, not durable). The deletion list omits `tests/conformance/fixtures/invariants/inferred-tear_candidates-unique-pk/` | Evidence cannot be reproduced from a clone, and a stale fixture survives | Cite the pinned `pyomo==6.10.1` distribution (byte-identical, §1). Add the fixture directory to the deletions | `just codegen-check` and `just governance` with zero failures after the deletion package |

### 12.4 Disposition of the §7 findings

| §7 finding | Disposition |
|---|---|
| 1 (contradicting normative text) | Resolved (§12.1) |
| 2 (connection-equation authority, duals, scaling), 3 (SD outcome contract), 6 (network tables) | Out of scope; carried by R-32 |
| 4 (policies without a home) | `tear_cost` check resolved; `fixed` removed; guesses out of scope (SD); stage policy remains as R2 |
| 5 (scope and parity claims) | Resolved: parity is claimed for selection only; the `networkx` pin is stated with pre-flight refusal; Slice B no longer depends on SD |
| 7 (determinism and keys) | Resolved; the new reuse-key gap is R1 |
| 8 (MILP) | Resolved: one platform MILP, termination check, 0.5 readback. Residue in R4 |
| 9 (cycle budget) | Resolved in principle; residue in R4 |
| 10 (consequences and evidence) | petgraph now treated honestly; residue in R6 |

**Commands for this re-review.** The failure baseline is zero.

| Command | Mode | Result | Failures |
|---|---|---|---|
| The 11 greps and `git diff -U0` of §12.1 | read-only text checks on the working tree | as tabulated | 0 contradictions; 2 gaps (R2, R4) |
| `just adr-lint` | ADR front matter, index and register lint | exit 0: `adr-frontmatter-check`, `adr-index-check` and `register-lint` passed | 0 |
| `just lint-typos` (after appending this section) | repository spell check | exit 0 | 0 |

No code changed and no tests ran. The scoped package is **Proposed** (not implemented).
Its Pyomo basis remains **Interface-checked** (§9, probes T1 and T2: `select_tear_heuristic`
and `check_tear_set` on a plain `MultiDiGraph`).
