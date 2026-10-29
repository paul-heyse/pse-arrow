# Design principle audit

This is the living diagnostic account of design-principle alignment in the implemented simulator
and its development tooling. Start with the [integrated system account](system-architecture.md)
and follow its six causes to the primary findings on the topic pages.

## Whole-system refresh — 2026-10-29

**Conclusion.** Plan56 corrected most of the preceding account. Its covariance-permission,
compiler-target, settlement, capability, closure and backup findings are resolved at current
source. Plan58 has since rebuilt large parts of the system in Rust.

The current misalignment is concentrated in six integrated causes:
1. scientific permission is owned for only part of what the system derives;
2. the orchestration crate interprets and composes what lower owners determine;
3. declared vocabularies, identities and transitions are applied by convention at consumers;
4. durable work pays a generic, fine-grained protocol per study point;
5. placement and generation follow build reach rather than responsibility, and machinery outlives
   its consumers;
6. the cutover moves entrypoints rather than responsibilities, so host policies have two live
   authorities.

The mathematics layer, typed solver outcomes, library-first numerics, store-side guarded
progression and the native host's ownership model are real strengths. The
[system account](system-architecture.md#six-integrated-causes) explains how the six causes
interact.

## Baseline and method

- **Audited source:** `f83bac0399d1d815b6ce91b692158b965fa054c7`, 2026-10-29, plus the working
  tree.
- **Included uncommitted changes, by path class:**
  - `pse-host` agent hooks, doctor, diagnostic tools and telemetry service (untracked when read);
  - the deletion of the Python generators from `pse-codegen`/`xtask/src/codegen`;
  - new `xtask/src/development/*` modules;
  - hook wiring in `AGENTS.md`, `.claude/settings.json` and `.codex/hooks.json`;
  - the justfile, Cargo manifests and a few runtime test files.
- **Also dirty:** `pse-runtime/src/host_operations/project.rs`, which removes the PyO3 method
  projection (DA-015).
- **Reconciliation:** HEAD advanced to `fbf08a43c` during the engagement.
  - `f3a05e0be` records the `pse-host` modules the streams read as untracked.
  - `fbf08a43c` adds a native boundary acceptance test target. `pse-runtime/Cargo.toml:69` is
    unchanged.
  - Decisive files for the other findings were unchanged.
- **Not included:**
  - concurrent edits to the Plan58 companion documents;
  - the deletion of `scripts/thermodynamic_campaign.py`;
  - later uncommitted edits to `pse-runtime/src/math/portable.rs` and
    `pse-buildinfo/src/identity.rs` that remove the Python producer role. Those bear on DA-024
    and are noted there.
- **Publication:** a later commit, recorded by Git. The inspected source baseline is distinct from
  it.
- **Input account:** whole-system source `682fc3830` (2026-10-25). Its scoped refreshes are the
  study and database boundary at `64a217de2` and T01 host foundation at `50f07ea6f`, published at
  `8fed0a30c`. Earlier text is recoverable there.
- **Standard:** Core 3.4; efficiency heuristics 1.0; process-simulator profile 1.5; pse-arrow
  binding ([standard.toml](../design_principles/standard.toml)). Test-assessment 1.2 applied only
  where tests were read as assurance; the [test account](../test-audit/README.md) owns test
  findings.
- **Scope:** the whole production system and development tooling:
  - all crates;
  - `python/pse` where it is still a supported path;
  - `xtask` and the generators;
  - the justfile and the scripts still reached from it.

  Retiring Python mechanisms were judged structurally: does each still hold live authority, and
  does the native replacement inherit its cause.
- **Maintainer premises retained:**
  - canonical SurrealDB/MX;
  - up to 16 SurrealDB-coordinated lanes;
  - durable work that outlives its caller, without a daemon;
  - uncapped ordinary builds;
  - CLIs and MCP optional, PyO3 deferred;
  - no library refused.

  These do not exempt their realization from assessment.

**Method.**
1. **Architecture-as-built map.** The coordinator first mapped the system from the
   architecture overview, ADR-0233, Plan58 and its companions, the workspace dependency graph and
   five end-to-end journeys.
2. **Six unanchored streams.** Fresh read-only design reviewers, who did not read the prior
   account, each took one cross-owner system question:
   - U1: scientific meaning and authority;
   - U2: numerical execution and solver integration;
   - U3: durable state and retained data;
   - U4: structure and composition;
   - U5: host and tooling;
   - U6: cross-cutting contracts.
3. **Revalidation.** A seventh stream (R1) revalidated every DA identity against current source.
4. **Gap streams.** The coverage check found two owners only touched in passing, so two more
   unanchored streams covered them:
   - U7: the declaration and data layer;
   - U8: the model preparation pipeline.
5. **Verification and synthesis.** The coordinator verified each consequential claim at its
   decisive source before adopting it, merged candidates by structural cause, and checked them
   against decision-closed findings. For example, U8's parallel reuse trackers fall within
   DA-030's decision.
6. **Challenge.** A fresh challenger reviewed the drafted account ([below](#challenge-reconciliation)).

**Evidence limits.** This is static architectural diagnosis, labelled **Interface-checked**.
- No product tests, solver runs, services, fault injection, benchmarks or numerical reference
  campaign ran.
- Read-only probes: `cargo metadata`/`cargo tree --offline --locked`, git history, grep/rg, and
  an approximate static Python import walk.
- Counts come from those searches and are lower bounds or approximations where noted.
- Cost and contention statements are structural and unmeasured.
- No overall alignment certificate is issued.
- `just turn-end`/`hygiene` were not run; audits do not own repository health.

## New findings

| ID | Finding | Primary home | Integrated cause |
|---|---|---|---|
| [DA-048](semantic-model-and-authority.md#da-048) | Only fit statistics have an owned product permission; other derived products are qualified by their consumers | Semantic model | 1 |
| [DA-049](semantic-model-and-authority.md#da-049) | Study, review and portability boundaries summarize permission in lossy, unenforced vocabularies | Semantic model | 1 |
| [DA-050](structure-and-invariants.md#da-050) | Identity derivations are catalogued per frame but minted inline at consumers through uncatalogued sub-contexts | Structure | 3 |
| [DA-051](structure-and-invariants.md#da-051) | Canonical transition authority is split across four representations and invoked through hand positional strings; settlement omits the module-hash binding | Structure | 3 |
| [DA-052](boundaries-and-composition.md#da-052) | Durable and service documents outside the exported set re-implement versioning and interpretation by hand | Boundaries | 3 |
| [DA-053](boundaries-and-composition.md#da-053) | Analysis modes compose native admission, routing, execution control and lane custody themselves | Boundaries | 2 |
| [DA-054](boundaries-and-composition.md#da-054) | Local and durable studies orchestrate the same occurrence semantics in two executors | Boundaries | 2 |
| [DA-055](execution-fit.md#da-055) | Each study point runs the generic durable lifecycle as many guarded commits against shared per-problem keys | Execution fit | 4 |
| [DA-056](execution-fit.md#da-056) | Retention safety is spread across many owner-specific protection families | Execution fit | 4 |
| [DA-057](development-tooling.md#da-057) | Native replacements move entrypoints one consumer at a time, so host policies keep a second live authority or interpreter | Development tooling | 6 |
| [DA-058](development-tooling.md#da-058) | The edit guard depends on an explicitly built executable; its protected-path set is hand-maintained in several places | Development tooling | 6 |
| [DA-059](boundaries-and-composition.md#da-059) | The declaration pipeline emits every projection for every relation family; retiring consumers anchor part of it | Boundaries | 5 |
| [DA-060](boundaries-and-composition.md#da-060) | Data-layer mechanisms and declared invariants outlive their consumers | Boundaries | 5 |

The next available identity is **DA-061**. Retired identities are never reused.

## Revalidation outcomes

| ID | Outcome at `f83bac039` | Primary home |
|---|---|---|
| DA-001, 002, 004 | Resolved (2026-10-25); stubs kept | [Semantic](semantic-model-and-authority.md#resolved-merged-and-moved-identities) |
| [DA-003](semantic-model-and-authority.md#da-003) | **Reopened**. The lookup rule is still authored in four places | Semantic |
| DA-005 | Resolved (physical context projected) | Semantic |
| [DA-006](semantic-model-and-authority.md#da-006) | **Reopened and re-scoped**. Obligation lowering, fixture policy and the omnibus specialized product | Semantic |
| [DA-007](structure-and-invariants.md#da-007) | **Reopened** against Plan56's resolved disposition and narrowed. Run-key prefixes; hand SurrealQL subsets; hand inverse projection | Structure |
| DA-008, 010, 011 | Resolved (2026-10-25) | Structure |
| DA-009 | Resolved (per-kind request facts; tagged replay) | Structure |
| DA-012, 013, 016 | Resolved or merged (2026-10-25) | Boundaries |
| DA-014, 018 | Closed by decision; not re-examined | Boundaries |
| DA-015 | Premise largely retired (MCP, CLI generator and catalogue deleted); `pse-py` residue retires under 58g G03 | Boundaries |
| DA-017 | Resolved. Non-algebraic selection is part of DA-053 | Boundaries |
| [DA-019](boundaries-and-composition.md#da-019) | Narrowed; live on a retiring path (Python supervisor is the operator store path) | Boundaries |
| [DA-020](boundaries-and-composition.md#da-020) | **Reopened** against Plan56's resolved disposition. Relocated to the host root and foundations | Boundaries |
| [DA-021](local-reasoning-and-assurance.md#da-021) | **Reopened** against Plan56's resolved disposition and widened (runtime breadth, cycles, visibility) | Local reasoning |
| DA-022 | Closed by decision (I16) | Execution fit |
| [DA-023](local-reasoning-and-assurance.md#da-023) | **Reopened and widened**. Sections omit the host layer and lag Plan58 ownership | Local reasoning |
| [DA-024](local-reasoning-and-assurance.md#da-024) | Live on a retiring path (producer qualification reachable only from `pse-py`) | Local reasoning |
| [DA-025](execution-fit.md#da-025) | Retained as a scope record | Execution fit |
| DA-026–029 | Resolved (2026-10-25) | Execution fit |
| DA-030 | Closed by decision (I08–I10); candidate trigger deleted; no stale reuse found | Execution fit |
| DA-033 | Resolved (shared exact settlement) | Boundaries |
| [DA-034](execution-fit.md#da-034), [DA-035](execution-fit.md#da-035) | Narrowed; per-point consequence absorbed by DA-055 | Execution fit |
| DA-036 | Resolved for authority; residue merged into DA-057 | Development tooling |
| DA-037 | Merged into DA-057 | Development tooling |
| DA-038 | Resolved for covariance; uncovered derived products are DA-048 | Semantic |
| [DA-039](structure-and-invariants.md#da-039) | **Reopened** against Plan56's resolved disposition and widened to canonical, native-service and host errors | Structure |
| DA-040 | Resolved (enum-keyed metrics) | Structure |
| DA-041 | Resolved (truthful vendored delta record) | Local reasoning |
| DA-042 | Resolved | Execution fit |
| [DA-043](development-tooling.md#da-043) | **Reopened** against Plan56's resolved disposition. Inherited by native entrypoints (per-invocation revalidation and executable digests) | Development tooling |
| DA-044 | Resolved; mechanism on a retiring path | Execution fit |
| DA-045 | Resolved | Execution fit |
| [DA-046](study-execution-and-database-boundary.md#da-046) | Inherited and corrected by the native CLI; `pse-py` path retiring | Study/database |
| [DA-047](study-execution-and-database-boundary.md#da-047) | Retained | Study/database |

DA-031 and DA-032 remain resolved or merged as recorded on the semantic page. Every identity keeps
its anchor on its page.

## Principle coverage

Each principle was assessed explicitly by at least one stream. The coordinator verified the gap
outcomes. "Aligned" means no structural gap was found in the inspected scope; it is not
certification.

| Principle | Outcome | Where |
|---|---|---|
| AP-01 Separation of concerns | Gap | Causes 2 and 5: DA-020, DA-021, DA-006, DA-053 |
| AP-02 Stable contracts | Gap | DA-049, DA-051, DA-052, DA-053, DA-060 |
| AP-03 Composition | Gap | DA-006, DA-053, DA-054. Host boundary composition aligned |
| AP-04 Domain model and authority | Core chain aligned; gaps at derived products and seams | DA-048, DA-003, DA-051, DA-039, DA-023 |
| AP-05 Explicit structure | Gap | DA-053 (fail-open scopes), DA-007, DA-058, DA-060 |
| AP-06 Local reasoning | Gap | DA-021, DA-053, DA-047. Cancellation and effect predicates are strengths |
| AP-07 Execution fit | Gap (structural, unmeasured) | DA-055, DA-056, DA-059, DA-020, DA-043 |
| DP-01 One authority per fact | Gap | DA-003, DA-006, DA-007, DA-048, DA-050, DA-057 |
| DP-02 Distinctions are types | Gap | DA-039, DA-049, DA-052, DA-006 (fixture policy) |
| DP-03 Invariants have enforcement points | Gap | DA-049, DA-051, DA-058, DA-060 |
| DP-04 Layered identity | Gap | DA-050; pointer identity in a reuse key (within DA-030's decision) |
| DP-05 Definitions/bindings/results separate | Mostly aligned | Value overlays, `PreparedCase::rebind`. Derived-product permission is not separated per product (DA-048) |
| DP-06 Author structure once | Partial | One registry declaration; generic accumulators. Canonical transitions are authored in four representations (DA-051) |
| DP-07 Relationship kinds | Aligned (not deep) | Connections separate from generated rows |
| DP-08 Operations carry contracts | Partial | `SpecializedModel` has no consumer contract (DA-006); DA-047 |
| DP-09 One reuse mechanism | Within a decision; one gap | Several premise/key trackers beside Salsa (DA-030 closed by decision, no stale reuse found). Identity re-minting threatens key consistency (DA-050) |
| DP-10 Prepare once, cross coarsely | Gap | DA-055, DA-043; preparation itself aligned |
| DP-11 Precision and determinism | Aligned | Fixed formal-symbol pool; no `target-cpu=native` |
| DP-12 Recursion and iteration semantics | Not assessed in depth | Strategy driver and recycle owners only skimmed |
| DP-13 Library first | Aligned in numerics; gap in data layer | Solver/numerics strengths; DA-060 engine wrapping; DA-052 hand JSON beside `cargo_metadata` |
| DP-14 Built-ins and thin adapters | Mostly aligned | Direct DataFusion providers; DA-060 |
| DP-15 Qualify library boundaries | Mostly aligned | Capability records for algebraic adapters. Non-algebraic selection is owner-decided (§18.1, ADR-0209) |
| DP-16 Minimize bespoke machinery | Gap | DA-053, DA-057, DA-059, DA-060 |
| DP-17 Module boundaries follow ownership | Gap | DA-020, DA-021, DA-053 |
| DP-18 Declared effects | Partial | Pure compiler and modeling; ambient thread-local premise capture; reads write tickets (declared) |
| DP-19 Owned workspaces and outcomes | Aligned on truthfulness; gap in proportion | DA-055, DA-056 |
| DP-20 Bounded, coordinated concurrency | Partial | One pool; split admission ledgers (DA-057); shared hot keys (DA-055) |
| DP-21 Structured diagnostics | Gap | DA-039; DA-024 and DA-047 (producer and attribution lineage) |
| DP-22 Labelled claims | Aligned (sampled) | Plan and ADR evidence labels; not a code property |
| DP-23 Verification matches risk | Test account | See [test account](../test-audit/README.md) |
| DP-24 Explicit, discoverable evolution | Gap | DA-023, DA-050, DA-051, DA-052, DA-059, DA-060 |
| PS-01 Dimensioned quantities | Aligned | Quantity contracts on ports, `CasePlan` slots, covariance reuse |
| PS-02 Material systems and validity | Aligned; one gap | Envelope and applicability owners; fitted-parameter publication leaves applicability unchanged (DA-048) |
| PS-03 Conservation | Aligned in modeling; gap in lowering | DA-006 |
| PS-04 Well-posed before solving | Aligned; minor | One structural analysis; backend recomputes the mode rule twice (`structural.rs`) |
| PS-05 Topology, structure and solve order distinct | Aligned | `pse-structural` split |
| PS-06 Formulation domain and smoothness | Aligned | Original guards, §7.3 owner |
| PS-07 Derivatives and scaling | Derivatives aligned; scaling ownership gap | DA-006 |
| PS-08 Initialization and convergence | Aligned (light) | Compiler-owned start precedence; strategy driver |
| PS-09 Solver selection by class | Aligned for algebraic classes | Explicit no-fallback routing. Non-algebraic selection is owner-decided (§18.1, ADR-0209), not assessed as a gap |
| PS-10 Truthful outcomes | Aligned for solves; gap for operational failures | DA-039 |
| PS-11 One model across modes | Partial | DA-053, DA-054 |
| PS-12 Honest results | Gap | DA-048, DA-049 |
| PS-13 Shared checks and reference validation | Aligned | Shared closure, applicability and conformance checks. Reference coverage is the test account's |

## Challenge reconciliation

A fresh read-only challenger reviewed the full draft against current source and the owner
documents. It raised 19 challenges. The coordinator verified the material ones at source and
integrated all of them:

| Challenge | Reconciled result |
|---|---|
| DA-020 misread `pse-model`'s 27k generated runtime family as canonical rows, and overstated host reach | Canonical rows are about 8.3k lines and consumed by no mathematics crate. The normal dependents of `pse-host` are `pse-runtime` → `pse-py`; `pse-operations` reaches it only in service/test builds. Revision 311's "heavy developer composition stays under xtask" cited as owner intent |
| DA-051 treated ADR-0164's designed writer-trust boundary as an unnoticed gap | Settlement trust is recorded as decided, open to a later review challenge. The retained gap is that `observe` omits the study-module hash binding that claims carry. About 300 hand SurrealQL statements added to the footprint |
| DA-055 omitted the live owner of its granularity | 58d D03 (O01), the state-placement table and §19.3's capture-gate rationale cited. The 10k and 100k bounds treated as declared limits. The cause is restated as a challenge to the selected unit |
| DA-053 called ADR-0209's decided fitting selector a gap, and said "no owner composes" | Non-algebraic selection is withdrawn as owner-decided. Restated: twelve literals, seven inside `MathService` and five in modes that bypass it |
| DA-023 claimed the sections never mention the host | Narrowed: §21.7 describes a Proposed host root, and the §1.1/§3.2 inventories omit the crate. `pse-admission`'s absence from §1.1 added |
| DA-059 claimed 58g names none of the projections; mixed in JSON Schemas; inconsistent counts | 58g's projection-consumer row cited; the gap narrowed to per-family emission declarations. JSON Schemas moved out. Counts restated with method and limits |
| DA-048 ignored the gating of own-prediction propagation and §19.2's product-local validity design | Narrowed to: modeling-step propagated covariance without recorded permission; sensitivities and Hessians by consumer conjunction; no relational projection of governing permission |
| Lineage labels silently contradicted Plan56's "resolved" dispositions | DA-007, 020, 021, 039 and 043 are now labelled reopened against Plan56, like DA-003, 006 and 023 |
| DA-057 called duplicate interpreters "authorities", and missed Plan58 I01/T02 | Rows split into writable authorities and duplicate interpreters. I01 ("no bridge") and T02 added. Reconciled with DA-036's resolution |
| DA-043 sat under the cutover cause; DA-003's fourth site was an identity encoding | DA-043 moved to bounded findings. The `f_<hex>` identity's primary home is DA-050 |
| DA-049 path ambiguity; DA-054's non-sequitur and misquoted ADR-0190 | Path qualified; divergence sentence dropped; ADR-0190 quoted exactly |
| DA-058 undercounted the protected-path copies and missed a false AGENTS.md claim | At least seven copies listed. The deny-list overstatement added. The hook exit contract cited |
| Quantities (runtime lines, host share, `pse-env` uses, terminal literals) | Qualified: about 259k non-test runtime lines; host artifact code about 40% of about 19k; 119 mentions of `pse-env`, 79 of them wrappers; about 25 hand terminal literals |
| The coverage table disagreed with finding principle lists (DP-05/06/09/17/21/24) | Rows reconciled |
| Architecture cited by line number; baseline text inconsistent; stale justfile line | Section identifiers used. Baseline records `fbf08a43c` and the extra dirty paths. Line re-anchored |

The challenger confirmed the core citations of DA-048…DA-060 and of the reopened findings. It also
confirmed the revalidation spot checks (DA-017, DA-033, DA-045) and that Plan58 cites no DA
identity. Its omission leads were integrated: `pse-admission` coverage, hand SurrealQL statements
and instruction drift. No challenge was resolved by product mutation or a qualification claim.

## Scope, coverage and limits

**Inspected, by stream (depth varied; each stream lists its own read and skimmed owners):**

| Area | Owners |
|---|---|
| Scientific meaning | Completion, run, fitting, uncertainty, studies, inspection, portability, research publication |
| Numerical execution | Backend execution seam and routing, structural admission, runtime solves, strategy, shooting, fitting oracles |
| Durable state | Canonical execution, studies and progression, migration, files, generated SurrealQL, worker, durable, study |
| Structure | The whole workspace graph, runtime module tree, codegen/xtask generators, host and operations edges |
| Host and tooling | All of `pse-host`, justfile routing, `scripts/pse-env`/`pse-host`, sampled Python policy scripts, `.config` |
| Contracts | `pse-ids`, `pse-diagnostics`, error enums, boundary documents, section truthfulness |
| Declaration and data | Codegen, `pse-schema`, `pse-relations`, `pse-engine`, `pse-rules`, `pse-columnar` consumers |
| Preparation | `pse-modeling`, `pse-compiler`, `pse-quantity`, `pse-kernels`, runtime modeling and preparation |

**Not inspected, or only lightly:**
- `pse-admission` beyond its authoring-driver entry and physical admission plans. It joins
  `pse-engine`, `pse-rules`, `pse-relations` and `pse-modeling`;
- the sys crates beyond their manifests;
- `pse-math` assembly internals;
- `pse-authoring` parser internals;
- horizon, surrogate and `native_child` internals;
- the KINSOL and IDAS adapter bodies;
- the `pse-engine` admission internals;
- the `pse-relations` validators;
- canonical staging, project import and explanations;
- `xtask` evidence and knowledge;
- most Python script internals;
- the `pse-py` mapping in depth (retiring).

Not established anywhere:
- numerical correctness;
- property-envelope completeness;
- server-internal CPU work;
- latency, memory and contention magnitudes.

An area without a finding is not thereby aligned.

**Unresolved questions relevant to the remaining Plan58 scope:**
- Should propagated covariance, sensitivities and fitted-parameter publication carry an owned
  product permission? Should that permission be relationally readable (DA-048)?
- Which canonical transitions should the store decide itself, rather than accept from the caller?
  Should guarded functions have generated call bindings (DA-051)?
- What unit of guarded commit fits many-cheap-case durable studies under the coordination premise
  (DA-055)?
- Which F tools belong in the cold host root? Should product crates depend on all of it (DA-020)?
- Does ADR-0164's writer-trust model for occurrence observation and terminal sealing still fit?
  If so, should `observe` at least bind the study-module hash (DA-051)?
- Should each remaining cutover move a responsibility with every consumer, so that compatibility
  seams are not built (DA-057)?
- Which generated projections, engine capabilities and declared invariants have a consumer after
  `pse-py` retires (DA-059, DA-060)?
- When does the architecture amendment that names the host layer land (DA-023)?

**Disposition owner.** Plan56 owned DA-001…DA-045 and is complete. Eight of its "resolved"
dispositions are reopened here with surviving-cause rationale:
- DA-003, 006, 007, 020, 021, 023, 039 and 043.

Plan58 does not adopt DA findings, and DA-046…DA-060 have no disposition owner. Live Plan58 owners
overlap some of them without citing them:
- 58d D03 (O01) for DA-051 and DA-055;
- 58g G03 and revision 311 (`2232bfa02`) for the retiring-path records DA-015, DA-019, DA-024,
  DA-046 and DA-059. This account selects no owner, authorizes
no repair and starts no solution review.

## Reading and ownership

| Page | Primary findings |
|---|---|
| [Integrated system account](system-architecture.md) | Six causes, interactions, scenarios, foundation narrative, improvement directions |
| [Semantic model and authority](semantic-model-and-authority.md) | DA-003, DA-006, DA-048, DA-049 |
| [Structure and invariants](structure-and-invariants.md) | DA-007, DA-039, DA-050, DA-051 |
| [Boundaries and composition](boundaries-and-composition.md) | DA-019, DA-020, DA-052, DA-053, DA-054, DA-059, DA-060 |
| [Local reasoning and assurance](local-reasoning-and-assurance.md) | DA-021, DA-023, DA-024 |
| [Execution fit](execution-fit.md) | DA-025, DA-034, DA-035, DA-055, DA-056; closed DA-022/DA-030 |
| [Development tooling](development-tooling.md) | DA-043, DA-057, DA-058 |
| [Study execution and database boundary](study-execution-and-database-boundary.md) | DA-046, DA-047 |

This collection owns diagnosis. Reviews own alternative remedies, plans own scheduled work and
closure evidence, and architectural owners and ADRs own accepted design and rationale. The
[audit skill](../../../.codex/skills/design-principle-audit/SKILL.md) and the
[binding](../design_principles/binding/pse-arrow.md#design-principle-audit) govern the process.
Refresh in place, preserve identities and lineage, and keep earlier exact accounts in Git.
