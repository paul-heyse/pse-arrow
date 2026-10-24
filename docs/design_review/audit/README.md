# Design principle audit

This is the living diagnostic account of where the production system and its development tooling
are misaligned with the selected design standard, and how alignment can improve.

**Where to start.** Read the [integrated system account](system-architecture.md) first. It covers:
- the system as built;
- how it serves the simulator's functional target;
- five integrated causes and how they interact;
- the hypotheses the evidence refuted;
- representative change and failure scenarios.

The topic pages hold the primary findings.

**What this account does not do.** It does not select remedies, set priorities, authorize work,
or certify that unlisted areas are aligned.

**Who owns what.**
- The [audit skill](../../../.codex/skills/design-principle-audit/SKILL.md) owns the method.
- The [binding](../design_principles/binding/pse-arrow.md#design-principle-audit) owns invocation
  and follow-up routes.
- The separate [test account](../test-audit/README.md) diagnoses test architecture.

## Baseline and method

- **Audited source:** `cbf45ccc933b66192bd7ed5b2a77176e39d46c2e` (`main`), 2026-10-24.
- **Publication:** a later commit, recorded by Git.
- **Included uncommitted changes:** none.
- **Excluded changes:**
  - commits after the baseline, through `61874d813`;
  - concurrent uncommitted `scripts/capabilities/` edits.

  Those changes touch capability tooling, `scripts/python_extension.py`,
  `scripts/semantic_edits.py`, `scripts/validation.py` and documentation; product crates, Python
  and `xtask` are unchanged. Tooling is in scope, so cited tooling sources were read at the
  baseline revision.
- **Selected standard:**
  - core 3.4;
  - efficiency heuristics 1.0;
  - process-simulator profile 1.5;
  - the pse-arrow binding, through [`standard.toml`](../design_principles/standard.toml).

  The test-assessment companion was consulted only where production assurance seams touch
  tests.
- **Scope:**
  - The product.
  - Development tooling as a full audit subject (maintainer selection, 2026-10-24): the FastMCP
    endpoint, recipes and `pse-env`, the substrate supervisor, `xtask`, runtime tooling, hooks,
    generated knowledge, and evidence tooling outside the DTA scope.
- **Maintainer premises, applied as binding and not audited:**
  - SurrealDB is canonical;
  - 16-lane SurrealDB-coordinated parallelism;
  - durable work that outlives its caller and spans processes, without a daemon;
  - a single local operator.

  Recorded maintainer decisions are named where they govern a finding.
- **Prior baseline:** the first audit at source `a791c533e` (2026-10-21), published at
  `0530dcaa1`. The intervening 865 commits include:
  - the Plan 49 remediation of DA-001–DA-032;
  - Plan 51 (projects, branches, history, exchange, backup v4);
  - Plans 50, 52 and 53 (diagnostics, the agent interface, test architecture).

**Method.** The earlier accounts in this repository published the union of crate-slice
candidates. The maintainer judged that approach myopic: narrow items were misread for lack of
the owner's intent, and system-level opportunities were missed. This refresh worked from the
system inward.

1. **System map first.** Before any finding work, the coordinator built a map of the system as
   built from:
   - the architecture sections, read as declared intent;
   - the crate graph;
   - representative journeys from the Python and Rust entry points (steady solve, value-only
     sweep, dynamics and fit, a durable study that outlives its caller, result
     selection/analysis/read, and the tooling journey).
2. **Six read-only evidence streams.** Each was a system question that spans owners:
   - meaning and authority;
   - change locality and composition;
   - execution fit and lifecycle;
   - scientific contract and local reasoning;
   - development tooling;
   - reconciliation of DA-001–DA-032, treating Plan 49's closure claims as leads.

   Each stream returned candidate *structural causes*. For each cause it gave scenario
   consequences, decisive sources and the strongest legitimacy case, checked against the owning
   section or ADR. Size, similar code or an unused feature alone could not raise a finding.
3. **Coordinator reconciliation into integrated causes.** The coordinator re-read the
   load-bearing citations of every new or reframed finding. Findings say where that happened.
4. **Independent challenge.** A fresh reviewer tested the draft for omissions, misread intent,
   over-narrow findings and overclaims. See [Challenge reconciliation](#challenge-reconciliation).

**Evidence.**
- Nothing was built, tested, measured or executed during this refresh.
- Claims are *Interface-checked* or *Implemented* (design principles §D).
- Two existing *Measured* records are cited under their original conditions: Plan 48f I30, and
  the Plan 49 decision I01 investigation. Neither is promoted to a current measurement.
- Counts are `git grep` or regex estimates at the baseline.
- Each finding names what would settle its open part.

## Integrated synthesis

The scientific core is well governed:
- physical typing;
- guards;
- library-owned derivatives;
- conservation by construction;
- one pure completion owner;
- one run supervisor;
- one pure study policy.

Governance weakens toward the rim of the system: the store, exchange, host and tooling layers.
That is where Plans 50–53 added most of their code. Five causes explain the material findings:

| # | Integrated cause | Findings |
|---|---|---|
| 1 | Contracts beyond record shapes are hand-authored per owner, host and language | [DA-007](structure-and-invariants.md#da-007), [DA-033](boundaries-and-composition.md#da-033), [DA-015](boundaries-and-composition.md#da-015), [DA-019](boundaries-and-composition.md#da-019), [DA-036](development-tooling.md#da-036) |
| 2 | Durability is realized per semantic unit rather than per workload | [DA-034](execution-fit.md#da-034), [DA-035](execution-fit.md#da-035), [DA-025](execution-fit.md#da-025) |
| 3 | Product and tooling responsibilities are placed by machinery location | [DA-020](boundaries-and-composition.md#da-020), [DA-037](development-tooling.md#da-037), [DA-024](local-reasoning-and-assurance.md#da-024) |
| 4 | Analysis operations lack an owned contract, and workflow internals lack boundaries | [DA-009](structure-and-invariants.md#da-009), [DA-021](local-reasoning-and-assurance.md#da-021) |
| 5 | Consumers re-derive or lose decisions the core already makes | [DA-006](semantic-model-and-authority.md#da-006), [DA-003](semantic-model-and-authority.md#da-003), [DA-038](semantic-model-and-authority.md#da-038), [DA-017](boundaries-and-composition.md#da-017), [DA-039](structure-and-invariants.md#da-039), [DA-005](semantic-model-and-authority.md#da-005) |

A bounded authority finding, [DA-023](local-reasoning-and-assurance.md#da-023), records that
design intent is not a reliable single authority.

**In terms of the functional target:**
- the edit → re-solve loop's critical path is dominated by fixed per-unit publication cost
  (DA-034);
- studies are sound in the middle and unfit at their population ends (DA-035, DA-025);
- recycles are library-owned and well structured in the bounded sample;
- results are physically checked, but unreadable outside Rust (DA-005);
- failures are often classed by constructor rather than by meaning (DA-039).

**Cross-cutting pattern.** Several Plan 49 remedies introduced the authoritative product without
forcing consumers through it, so the old interpretation moved rather than disappeared. The
retained causes also grew with each later plan, because new capability lands in the same
generative loci. Site repairs therefore do not stop recurrence. The useful targets are the
generating mechanisms
([integrated account](system-architecture.md#cross-cutting-pattern-moved-causes-and-structural-attractors)).

## Material corrections to the previous account

**Plan 49's closure is not inherited.** Plan 49 marked all 32 findings resolved. At this
baseline, they fall into five groups:

| Group | Count | Findings |
|---|---|---|
| Resolved | 8 | DA-004, 008, 010, 011, 013, 026, 027, 028 |
| Resolved in their owner, with a residue merged into another finding | 5 | DA-001, 002, 012, 029, 031 |
| Closed by maintainer decision | 4 | DA-014, 018, 022, 030 |
| Merged into active findings | 2 | DA-016, 032 |
| Continuing as active findings | 13 | — |

Several of the 13 active findings are partly governed by decisions:
- DA-020 by I21;
- DA-024 by I24;
- DA-025 by the no-collector decision.

Four closure claims are contradicted by unchanged files:
- DA-003: `selected_source.rs` is byte-identical;
- DA-017: the KINSOL published row;
- DA-023: the named drift remains;
- DA-005: `forget_study_results(str)`.

Closure evidence that tested a delivered remedy cannot show that a cause is gone elsewhere.

**DA-025's original framing is withdrawn.** "Persistence machinery sized beyond actual
participants" does not hold under the binding premises. Its surviving content is split:
- the missing physical collection stays as DA-025;
- per-owner protocol re-authoring becomes DA-033;
- the publication unit becomes DA-034.

**Reframed findings:**
- **DA-006.** Its original core is resolved: per-mode lowering, duplicates and last-wins. The
  remaining runtime re-derivations share one cause and absorb the residues of DA-001, DA-002,
  DA-029 and DA-031.
- **DA-009.** Pairing is now structural. The missing per-kind contract absorbs DA-012's dispatch
  strand.
- **DA-019.** The second writer is gone. What remains is the deployment contract and the
  restore-comparison owner set living in tooling.

**Merges:**
- DA-005 is split. Physical meaning stays; canonical key kinds move to DA-007.
- DA-007 also absorbs the store-key half of DA-002, DA-032's artifact-profile literals and the
  lifecycle projections.
- DA-017 absorbs DA-016's residue.

**Closed by recorded maintainer decision, with revisit triggers on their pages:**
- DA-012: strategies stay nondurable;
- DA-014: I22, RC12;
- DA-018: I23;
- DA-022: I16;
- DA-030: I08–I10.

The per-problem creator guard was considered under I01 and not raised.

A withdrawn or narrowed diagnosis reflects corrected judgment or a code change established from
source. It is not a new test result.

## Challenge reconciliation

An independent reviewer challenged the draft against source at the baseline and against the
retired Plan 49 records. Every material challenge was accepted, and every one was reconciled:
- **C1: a draft finding on the per-problem creator guard contradicted decision I01.** Withdrawn
  and recorded under *Considered and not raised*, with what would reopen it (Plan 51 creators and
  KPI per-member protections postdate I01's measurement). The ID was never published, so the new
  IDs were renumbered contiguously.
- **C2: the edit → re-solve workload was untraced, and I30's measurement was missed.** DA-034 now
  leads with the interactive critical path. It cites I30 as *Measured* under its original
  conditions, notes the deliberate durable default and the ephemeral route, and names
  ADR-0187's packing rule as the rule impact. A scenario row was added.
- **C3: the account was not organized around the functional target.** The integrated account
  now has a functional-target section covering the five workloads and two result qualities. A
  bounded recycle (PS-08) sample was taken; it found a library-owned design and no new finding.
- **C4: DA-006 misread the compiler's product.** Reframed. The compiler publishes one
  mode-invariant classification. Steady mode duplicates it; dynamic mode fills a gap that no
  owner covers.
- **C5: settled decisions were kept as live defects.** DA-022 and DA-030 are now
  closed-by-decision rows with revisit triggers. "Never researched" was deleted.
- **C6: the backup claim was overstated.** DA-019 now states that backup copies and
  byte-verifies everything. Only the logical owner comparison is hand-listed and omits Plan 51
  tables. It was removed from the moved-cause list.
- **C7: two causes were conflated, and findings sat under two causes.** Accepted:
  - the contract halves of the old store, tooling and operation causes became one spine
    (cause 1);
  - DA-017 and DA-038 moved to cause 5;
  - DA-005 was split;
  - each finding now has one cause.
- **C8: narrow items.** Accepted:
  - DA-038 (formerly the permission finding) lost its store projections to DA-007;
  - DA-003 is linked into DA-006;
  - DA-024 sits under cause 3.

  DA-003 keeps its own scope because its modeling-side inventory is a distinct owner.
- **C9: headline counts.** Corrected to the breakdown above. Plan 51 accounts for 39 of the 55
  new functions.
- **C10: a strength contradicted DA-038.** The strength is narrowed, and the LM route is treated
  as top-level.
- **C11: DA-039's counterevidence and clearest case.** Added the roughly 145
  `ProblemError::internal` sites and the timeout/resource-limit persistence path.
- **C12: DA-008's residue.** The free-text THROWs in the development stores are noted as a
  DA-020 manifestation.
- **C13: name governing decisions.** Each finding names the decision it would reopen:
  ADR-0187, I28, I21, I20, I01, I16, I08–I10.
- **C14: "supported size".** Now "the admitted bound", noting that point rows are already
  batched.
- **C15: cause 6 overclaimed.** Demoted to a bounded authority finding. ADR-0214 is described as
  replacing ADR-0164's outcome without a supersession link.
- **C16: excluded changes and labels.** Corrected. "Hypothesis" is not used as an evidence
  label.

The challenger confirmed:
- the publication and file mechanics;
- the study loops;
- the missing collection caller;
- the research-settlement divergence;
- the duplicated resource values;
- the tooling import cycle;
- the failure-class chain;
- the KINSOL record;
- the ADR and drift facts;
- the table and function counts;
- the DA-010, DA-026 and DA-027 resolutions.

## How the standard informed this refresh

**AP-01 — separation of concerns.** The question was which owners change for the binding
scenarios and for the Plan 51 change. The core localizes change well; PSE-S01 is package-only.
The runtime is a legitimate workflow owner, but its flat namespace gives its owners no boundary
(DA-021). Product and tooling responsibilities follow where machinery lives (cause 3).

**AP-02 — stable contracts.** Solver adapters are a strong seam; published capability is the weak
point (DA-017). Hosts depend on hand-projected operation lists (DA-015). The deployment contract
is undeclared (DA-019).

**AP-03 — composition.** One run supervisor and one study policy show composition working. Per
analysis kind, per durable owner and per host, the same questions are answered again instead of
composed (DA-009, DA-033, DA-015).

**AP-04 — domain model and semantic authority.** Model adequacy and authority were assessed
separately:
- the core model is adequate and governs behavior;
- the canonical layer's model describes record shapes while meaning lives in strings (DA-007);
- physical meaning is lost at the public boundary (DA-005);
- permission and classification are re-derived by consumers (DA-038, DA-006);
- design intent has several writable statements per fact (DA-023).

**AP-05 — explicit structure.** Store-enforced attempt transitions are a strength. Failure
meaning follows the constructor rather than a declaration (DA-039). Workload classes and budgets
are remembered conventions (DA-036).

**AP-06 — local reasoning.** Pure policy cores exist and are consumed. Formulation, horizon and
profile policy remain in effectful drivers (DA-021, DA-038). Tooling owners sit in one import
cycle with the production supervisor (DA-037). Test-architecture consequences belong to the DTA
account.

**AP-07 — execution fit.** Assessed qualitatively for edit/re-solve, studies, dynamics and
concurrency, with H2, H7, H8, H11 and H22–H28 most relevant. The interactive critical path, the
population ends of studies, the publication unit and the missing collection are the structural
gaps (DA-034, DA-035, DA-025). The per-problem exclusion was weighed and left to decision I01.

**Operational refinements:**
- **Meaning and identity (DP-01–07)** drove causes 1 and 5 and DA-023.
- **Transformation and reuse (DP-08–12):** DA-006 and DA-038. DP-09 reuse keys were not
  re-examined beyond DA-030.
- **Libraries and ownership (DP-13–17):**
  - libraries are contained;
  - FastMCP and DataFusion are used natively;
  - the bespoke duplications are the per-owner protocols, the liveness rules and the producer
    chain (DA-033, DA-037, DA-024).
- **Effects, lifecycle and diagnostics (DP-18–21):** DA-034, DA-035, DA-025 and DA-039.
- **Evidence and evolution (DP-22–24):** DA-023, and the versioned exchange documents under
  DA-007.

**Process-simulator profile:**
- *Assessed in detail, mostly as strengths:*
  - PS-01 (with the boundary loss in DA-005);
  - PS-03;
  - PS-04 (with an attribution gap in DA-039);
  - PS-06;
  - PS-07 (scaling classification in DA-006);
  - PS-09 (DA-017);
  - PS-10 and PS-12 (DA-038);
  - PS-11 (DA-006).
- *Assessed in part:*
  - PS-02, at the evaluation-time gate;
  - PS-05 and PS-08, through tear selection, causal maps and §17.4, in a bounded sample;
  - PS-13, through shared conformance fixtures.

## Revalidation outcomes

| ID | Outcome | Current scope |
|---|---|---|
| DA-001 | Resolved; residue merged | The typed resolved IR owns syntax; runtime name lookup → [DA-006](semantic-model-and-authority.md#da-006) |
| DA-002 | Resolved; residue merged | Modeling derivations catalogued; runtime re-mints → DA-006; store keys → DA-007 |
| [DA-003](semantic-model-and-authority.md#da-003) | Narrowed | Selected-source inventory and two lexical rules restate roles |
| DA-004 | Resolved | Generated artifact and owner enums |
| [DA-005](semantic-model-and-authority.md#da-005) | Narrowed, reframed, split | Physical meaning unreadable at the public boundary; key kinds → DA-007 |
| [DA-006](semantic-model-and-authority.md#da-006) | Reframed and widened | Runtime formulation re-derives or fills in published classification, identity and roles |
| [DA-007](structure-and-invariants.md#da-007) | Retained and widened | Canonical store meaning, keys and projections outside the registry, extended by Plan 51 |
| DA-008 | Resolved | Typed store refusals, effects and worker loss |
| [DA-009](structure-and-invariants.md#da-009) | Reframed | Pairing is structural; there is no owned per-kind operation contract |
| DA-010 | Resolved | Typed covariance origin, verified fit, PSD external admission |
| DA-011 | Resolved | Generated document catalogue; fingerprint check |
| DA-012 | Resolved; residue merged | One run supervisor; dispatch strand → DA-009 |
| DA-013 | Resolved | One pure prediction policy |
| DA-014 | Closed by decision | Parameter-valued points; other kinds refused (I22, RC12) |
| [DA-015](boundaries-and-composition.md#da-015) | Narrowed and widened | Modeling chain fixed; Plan 51 hand-projected into four hosts |
| DA-016 | Narrowed and merged | Residue → [DA-017](boundaries-and-composition.md#da-017) |
| [DA-017](boundaries-and-composition.md#da-017) | Retained | Published capability disagrees with admission; runtime names backends |
| DA-018 | Reframed; closed by decision | Integration patch (I23); size-scope gap → DA-023 |
| [DA-019](boundaries-and-composition.md#da-019) | Reframed | Deployment contract and restore-comparison owner set live in tooling |
| [DA-020](boundaries-and-composition.md#da-020) | Retained and widened | Tooling families in the registry, model and store client (runtime module kept by I21) |
| [DA-021](local-reasoning-and-assurance.md#da-021) | Retained | Runtime-carrying values; flat workflow namespace |
| [DA-022](execution-fit.md#da-022) | Closed by decision (I16) | Hand size models; revisit trigger recorded |
| [DA-023](local-reasoning-and-assurance.md#da-023) | Retained and grown | Design-intent authority |
| [DA-024](local-reasoning-and-assurance.md#da-024) | Narrowed and widened | Native child by decision (I24); strict producer chain without a consumer |
| [DA-025](execution-fit.md#da-025) | Reframed and narrowed | Physical collection has no production route |
| DA-026 | Resolved | Per-row locators removed; DataFusion pruning |
| DA-027 | Resolved | Group/re-expand cycle deleted |
| DA-028 | Resolved structurally | Multi-output guarded regions |
| DA-029 | Resolved in the compiler; residue merged | Runtime scans → DA-006 |
| [DA-030](execution-fit.md#da-030) | Closed by decision (I08–I10) | Hand-keyed cross-lane caches beside Salsa |
| DA-031 | Resolved; residue merged | Compiler view policy; inline role admission → DA-006 |
| DA-032 | Narrowed and merged | Artifact-profile literals → DA-007 |
| [DA-033](boundaries-and-composition.md#da-033) | New | Lifecycle envelope re-authored per owner |
| [DA-034](execution-fit.md#da-034) | New | Publication unit per semantic item, each with the full file protocol |
| [DA-035](execution-fit.md#da-035) | New | Study population operations as item loops |
| [DA-036](development-tooling.md#da-036) | New | Resource and workload meaning without an authority |
| [DA-037](development-tooling.md#da-037) | New | Process-ownership primitives re-authored; a cyclic web that reaches production |
| [DA-038](semantic-model-and-authority.md#da-038) | New | Result permission re-derived for aggregates, the LM route and composites |
| [DA-039](structure-and-invariants.md#da-039) | New | Failure class chosen by constructor, not meaning |

Each topic page keeps an anchor and a one-line record for its resolved, merged and closed
identities. DA-001–DA-039 are allocated, retired IDs are never reused, and the next available ID
is DA-040.

## Strengths to preserve

These bound the findings; a later remedy must not discard them. Each was re-verified at this
baseline.

- **Physical typing.** One owner holds the complete quantity type
  (`pse-quantity/src/quantity_type.rs`). Side inputs are quantity-keyed Arrow rows checked before
  custody (`workflow/physical_values.rs`).
- **Guards survive rewriting, and derivatives refuse rather than fall back** (`pse-math/src/guarded.rs`;
  `execution.rs::available_order`). No silent finite-difference path was found.
- **The applicability gate is enforced at evaluation.** Unpermitted out-of-envelope inputs raise
  `MathError::Applicability`.
- **Conservation by construction; PSE-S01 is package-only.**
- **Structural admission before solve.** Matching, Dulmage–Mendelsohn and block-triangular
  analysis through pounce-presolve, with refusal by semantic id.
- **Library-owned recycles.** A causal sweep is a KINSOL map. Tears come from an exact MILP or an
  explicit greedy route, with an independent acyclicity witness. Evaluation uses immutable
  overlays.
- **One pure completion owner.** `workflow/numerics.rs` is consumed by every run-level route
  except the automatic residual-LM fit. Summaries read the persisted decision. Qualification is
  backend-neutral (`pse-backend-native/src/quality.rs`).
- **One run supervisor and structural request/result pairing** (`RunSupervisor`, `RunOutcome`).
- **One pure study policy**, executed natively and in the store (`pse-model::study_policy`,
  `pse-study-module`).
- **The durable lifecycle's correctness core.**
  - No native work runs inside transactions.
  - Claims are fenced before effects.
  - Terminal admission checks against a frozen acknowledgement inventory.
  - Seed eligibility is sealed with completion.
  - Recovery is per attempt and never re-runs science.
  - Cancellation drains.
  - Attempt transitions are enforced by the store.
  - Refusals carry registry codes.
  - The per-problem creator guard is retained deliberately (I01).
  - Backup copies and byte-verifies the complete database.
- **Execution core.** Lane-level structural reuse. Compact output shaped by demand, which refuses
  rather than truncates. Publication lanes that overlap compute. DataFusion-planned result reads
  with pruning.
- **Library containment.**
  - The SurrealDB SDK, Salsa and Symbolica each have one owner.
  - FastMCP, systemd and `pyo3-introspection` are used natively.
  - Tooling reads generated schemas rather than mirroring them.
- **No science in Python.** Arrow streams cross at batch granularity, and decoding is strict and
  checked against fingerprints.

## Scope, coverage and limits

**Inspected.** Representative owners and journeys across all product crates and the Python
facade. On the tooling side:
- the FastMCP server core and native bindings;
- `pse-env` and host admission;
- the supervisor's deployment, backup and liveness paths;
- the `xtask` roots;
- the producer chain;
- runtime tooling placement.

**Not examined, or sampled only:**
- scientific and numerical correctness of the reference packages;
- recycle convergence history beyond §17.4;
- adapter internals for SCIP, HiGHS, Clarabel, IDAS, PETSc and Uno;
- compiler and Salsa reuse internals beyond the resolved sites;
- backup v4 and merge internals;
- most of the FastMCP inventory;
- `surreal_server.py` beyond the cited functions;
- the semantics of the vendored SurrealDB SDK and server patches;
- interplay between DataFusion and native thread pools.

**Unmeasured.** No current elapsed time, file count, contention, memory or capacity was
measured. Earlier *Measured* records keep their original conditions.

**Not proof of absence.** No finding is a statement that unlisted code is aligned.

## Reading and ownership

| Page | Contents |
|---|---|
| [Integrated system account](system-architecture.md) | System picture, functional target, the five causes, the authority finding, interactions, refuted hypotheses, scenarios, improvement directions |
| [Semantic model and authority](semantic-model-and-authority.md) | DA-003, DA-005, DA-006, DA-038 |
| [Structure and invariants](structure-and-invariants.md) | DA-007, DA-009, DA-039 |
| [Boundaries and composition](boundaries-and-composition.md) | DA-015, DA-017, DA-019, DA-020, DA-033 |
| [Local reasoning and assurance](local-reasoning-and-assurance.md) | DA-021, DA-023, DA-024 |
| [Execution fit](execution-fit.md) | DA-025, DA-034, DA-035; closed-by-decision DA-022, DA-030; the per-problem guard (considered and not raised) |
| [Development tooling](development-tooling.md) | DA-036, DA-037 |

The topic pages are organizational aids. A finding's principle IDs carry its criteria. Each
finding has one primary home across this account and the test account, and overlapping DTA
findings are linked rather than copied.

**Ownership.**
- This collection owns diagnosis at its stated baseline.
- Reviews own remedy alternatives and judgment.
- Implementation plans own scheduled disposition and closure evidence.
- Architectural owners and ADRs own accepted design.

**Previous consuming review:**
[in-use-library remediation review](https://github.com/paul-heyse/pse-arrow/blob/197616768b1b7abcbe67af2b5b8d42ab445b291b/docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md),
2026-10-21. It consumed the first account at `0530dcaa1`, and its dispositions are now history.

**Refresh rule.** Refresh in place before a new review engagement:
- preserve identities and lineage;
- remove resolved material once source establishes the correction;
- keep the earlier text and its dated evidence in Git and the removal commit.
