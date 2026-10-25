# Design principle audit

This is the living diagnostic account of where the production system and its development tooling
are misaligned with the selected design standard, and how alignment can improve.

**Where to start.** Read the [integrated system account](system-architecture.md) first. It covers:
- the system as built;
- how it serves the simulator's functional target, including the agent loop;
- five integrated causes and how they interact;
- the result of testing the structural-attractor prediction on Plans 52–55;
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

- **Audited source:** `b5bdac9da763bf1a70f20b0599de0edad8a955f9` (`main`, clean), 2026-10-25.
- **Publication:** a later commit, recorded by Git.
- **Included uncommitted changes:** none.
- **Excluded changes:**
  - the concurrent commit `0478f0990` (MCP acceptance tests and
    `docs/dev/mcp-capability-assessment-2026-10-25.md`);
  - the later concurrent uncommitted `scripts/tests/mcp_*_acceptance.py` edits.

  They touch test and documentation files only. Product crates, Python, `xtask` and tooling
  owners are unchanged.
- **Selected standard:** unchanged since the previous refresh:
  - core 3.4;
  - efficiency heuristics 1.0;
  - process-simulator profile 1.5;
  - the pse-arrow binding, through [`standard.toml`](../design_principles/standard.toml).

  The test-assessment companion (1.2) was consulted only where production assurance touches
  tests.
- **Scope:**
  - the product;
  - development tooling as a full audit subject (maintainer selection, 2026-10-24);
  - by maintainer selection on 2026-10-25, the areas the previous account named as unexamined:
    - solver adapter internals;
    - recycle convergence;
    - vendored SurrealDB SDK and server patches (and, as found, SUNDIALS);
    - compiler and Salsa reuse;
    - backup v4 and merge;
    - DataFusion/native thread pools;
    - most of the FastMCP inventory;
    - the supervisor beyond the cited functions.
- **Maintainer premises, applied as binding and not audited:**
  - SurrealDB is canonical;
  - 16-lane SurrealDB-coordinated parallelism;
  - durable work that outlives its caller and spans processes, without a daemon;
  - a single local operator;
  - the local uncapped build mode.

  Recorded maintainer decisions are named where they govern a finding.
- **Prior baseline:**
  - the previous refresh at source `cbf45ccc9` (2026-10-24), published at `2257311c0`. No review
    consumed it;
  - 131 intervening commits:
    - Plan 52 OpenTelemetry (process telemetry, owner metrics, Collector and OpenObserve
      services, W3C context through the vendored SDK);
    - Plan 54 build reuse (sccache, Cargo freshness, uncapped admission);
    - Plan 55 MCP effectiveness (native result inspection v2, ADR-0224; paired evidence
      comparison, ADR-0225; a 57-module capability plane and generated CLI);
  - canonical tables, server functions, workflow modules and per-kind arms are unchanged.

**Method.** This is a change-directed refresh plus the prior coverage gaps. It keeps the 10-24
refresh's system-inward method.

1. **System map first.** The coordinator re-read the prior map and the changed intent:
   - the §19, §23.1 and §24 diffs;
   - ADR-0224 and ADR-0225;
   - the interface guide.

   It then traced the new journeys through source:
   - process host → telemetry → services;
   - MCP/CLI → capability plane → native contracts → inspection and comparison;
   - build → `pse-env` → sccache.

   It also counted the attractor loci at both baselines.
2. **Five read-only evidence streams,** each a cross-owner question carrying its share of the
   prior gaps:
   - S1 meaning, authority and contracts;
   - S2 placement, composition and local reasoning, plus the compiler and Salsa;
   - S3 execution fit and lifecycle, plus backup, merge and thread pools;
   - S4 scientific contract and the PS profile, plus adapters, recycles and vendored patches;
   - S5 development tooling, plus the FastMCP inventory and the supervisor.

   Each stream returned candidate structural causes, with scenario consequences, decisive
   sources, the strongest legitimacy case checked against the owning section or ADR, and what
   would settle it. Each stream also revalidated its assigned identities.
3. **Coordinator reconciliation into the existing causes.** Two causes were restated more broadly
   where the evidence crossed their earlier boundary. The coordinator re-read the load-bearing
   citations of every new or widened finding, and each finding says where.
4. **Independent challenge.** A fresh reviewer tested the draft against source. See
   [Challenge reconciliation](#challenge-reconciliation).

**Evidence.**
- Nothing was built, tested, measured or executed against the product.
- Read-only probes were used: `git grep` counts at both baselines, `ast` import-graph scans,
  schema parses, and diffs of vendored trees against checksum-verified upstream archives
  downloaded or extracted to scratch.
- Claims are *Interface-checked* or *Implemented* (design principles §D).
- The one existing *Measured* record, Plan 48f I30, is cited under its original conditions.
- Each finding names what would settle its open part.

## Integrated synthesis

The scientific core is well governed. This refresh's first look at its internals added three
strengths to the list:
- the solver adapters (exhaustive status maps, original-space qualification, contained callbacks,
  analytic derivatives);
- recycle convergence;
- Salsa reuse.

Governance weakens toward the rim: the store, exchange, host, observability and tooling layers.
Plans 52–55 built almost entirely there. Five causes explain the material findings:

| # | Integrated cause | Findings |
|---|---|---|
| 1 | Contracts are declared as shapes; meaning, vocabularies and protocols are hand-authored per owner, host and language | [DA-007](structure-and-invariants.md#da-007), [DA-040](structure-and-invariants.md#da-040), [DA-033](boundaries-and-composition.md#da-033), [DA-015](boundaries-and-composition.md#da-015), [DA-019](boundaries-and-composition.md#da-019), [DA-036](development-tooling.md#da-036) |
| 2 | Fixed protocol cost is paid per semantic unit rather than per workload | [DA-034](execution-fit.md#da-034), [DA-035](execution-fit.md#da-035), [DA-025](execution-fit.md#da-025), [DA-044](execution-fit.md#da-044), [DA-043](development-tooling.md#da-043) (its per-call half) |
| 3 | Product and tooling responsibilities are placed by machinery location | [DA-020](boundaries-and-composition.md#da-020), [DA-037](development-tooling.md#da-037), [DA-024](local-reasoning-and-assurance.md#da-024) |
| 4 | Analysis operations lack an owned contract, and workflow internals lack boundaries | [DA-009](structure-and-invariants.md#da-009), [DA-021](local-reasoning-and-assurance.md#da-021) |
| 5 | Consumers re-derive or lose decisions the core already makes | [DA-006](semantic-model-and-authority.md#da-006), [DA-003](semantic-model-and-authority.md#da-003), [DA-038](semantic-model-and-authority.md#da-038), [DA-017](boundaries-and-composition.md#da-017), [DA-039](structure-and-invariants.md#da-039), [DA-005](semantic-model-and-authority.md#da-005) |

Three bounded findings sit beside them:
- [DA-023](local-reasoning-and-assurance.md#da-023): design intent is not a reliable single
  authority;
- [DA-041](local-reasoning-and-assurance.md#da-041): edited vendored trees have no truthful patch
  record, and their prose contradicts the bytes and §13;
- [DA-042](execution-fit.md#da-042): exported span granularity is a crate-prefix convention, and
  the SDK's drop count is compiled away.

DA-043's currency half is a bespoke reuse key held by the tooling (DP-09, DP-13). It resembles
cause 5 and is not forced under cause 2.

**In terms of the functional target:**
- The edit → re-solve loop's critical path is still dominated by fixed per-unit publication cost
  (DA-034).
- Studies are sound in the middle and unfit at their population ends. Backup of the resulting
  never-collected inventory runs under a fixed clock (DA-035, DA-025, DA-044).
- Recycles and solver outcomes are library-owned and truthful.
- The new agent loop is built on owned, kind-agnostic contracts. Each call pays a full
  lifecycle that its own plan target (LIFE04) ruled out, and any crate edit makes reads refuse,
  including supplied-only comparison (DA-043). What it shows is unitless and
  permission-unknown (DA-005, DA-038). Its failures carry an input remedy whatever their meaning
  (DA-039).

**The attractor test.** The prior account predicted that new capability lands in the same
generative loci. Plans 52–55 bore this out for causes 1–3: vocabularies restated around generated
shapes, hand-bound agent operations, a defeated feature gate, re-authored service liveness and a
per-call protocol. It failed where the repository already had and used a declare-once or
owned-contract mechanism: generated document shapes, Rust-declared metric names, a CLI derived
from MCP schemas, and inspection and comparison without per-kind arms. Those counterexamples are
the strongest evidence that the improvement directions are realizable
([integrated account](system-architecture.md#cross-cutting-pattern-moved-causes-and-structural-attractors)).

## Material corrections to the previous account

**Restated causes.**
- **Cause 1** now reads "declared as shapes", not "beyond record shapes". Plans 52–55 generated
  every new document shape, so shapes are no longer the gap. The gap is the vocabularies, bounds
  and instrument semantics inside them (DA-040).
- **Cause 2** now reads "fixed protocol cost per semantic unit", not "durability per semantic
  unit". The same structural form appears in backup verification, agent calls and exported spans,
  which are not durability.
- **Cause 3 keeps its title.** A draft restatement as "machinery reach" was evidenced only by
  DA-020, so it was kept as that finding's reframing (C15). Product observability now lives in
  the tooling module, and generator or host reach decides placement.

**DA-037's footprint was understated.** The previous account named six modules in the cyclic
import web. An `ast` probe that counts function-local imports finds the strongly connected
component held 26 modules at `cbf45ccc9`, and 27 now.

**Previous counting method.** The prior locus table's "canonical tables 62 → 79" and "server
functions 116 → 171" used a different method from this refresh's `*.surql` occurrence counts.
Those figures are kept as historical, and the new table uses one method at both baselines.

No prior finding was withdrawn, and none was resolved in this window.

## Challenge reconciliation

A fresh reviewer challenged the draft against source at `b5bdac9da` and the owning sections,
ADRs and plans. It raised 19 challenges, 11 of them material. Every challenge was accepted and
reconciled. The coordinator re-checked the decisive sources of C1, C3, C6, C10, C11 and C18.

- **C1: DA-043 missed owner intent.** Plan 52n LIFE04 explicitly forbids a whole-repository hash
  or unsound hand whitelist per read call, and requires the limit to be disclosed. The Plan 52
  freshness amendment and 52k MCP06 say the same. DA-043 now reads as the implementation
  diverging from its own recorded target, and the pooled-worker direction names the §24.1 and
  MCP06 wording it would amend.
- **C2: wrong ADR.** ADR-0219 governs evidence snapshots, not currency. The citation was replaced
  by §24.1 and LIFE04.
- **C3: cost undercounted.** `capture()` imports `pse` in a second interpreter and SHA-256s the
  whole extension binary. That makes five subprocesses, not three, and the tool binaries are
  hashed in full. Corrected.
- **C4: the CLI layer.** Each CLI invocation starts a fresh endpoint by design (55a). The
  selected JSONL session (55d D03) amortizes the endpoint but not the native child. Added.
- **C5: inconsistent currency semantics.** The knowledge executable is checked only for being
  executable, while supplied-only `evidence_compare` inherits the product-extension gate. Added
  to DA-043.
- **C6: DA-041's headline was wrong.** §13 declares the IDAS quadrature-history correction, §24.2
  records its qualification, and `idas_initial_tests.rs` pins it. The "undeclared numerical
  hunk" claim is withdrawn. DA-041 is narrowed to records that contradict the bytes and §13, with
  no hunk inventory and no R-41 entry.
- **C7: commit range.** Corrected to `d85408066`…`a791c533e` (2026-10-17 to 10-21).
- **C8: DA-041's legitimacy.** ADR-0189's withdrawal of provenance-proof targets is named. The
  direction now uses the existing patch-series form and R-41, with no new check.
- **C9: OTel and Diffsol.** No OTel family is declared, so `family-check` cannot see a split. The
  Diffsol trees were examined (build files only) and are now listed.
- **C10: DA-042 overclaimed.**
  - Per-request SDK spans are contract (Plan 52 O02/F02).
  - Possible loss is a declared §24.1 limit.
  - The RC01/RC02 characterization was wrong.

  Span volume moved into DA-034's consequence. DA-042 is narrowed to prefix-selected granularity
  plus the SDK drop count that is compiled away (`internal-logs` is off), and it moved out of
  cause 2.
- **C11: DA-044 and a false strength.**
  - §20 disclaims power-loss durability, so the missing database fsync is reframed as an
    inconsistency within a declared limit.
  - The strength "backup byte-verifies the complete database" was corrected. Only payload files
    are compared against source.
  - Added: a verifier timeout leaves the store frozen until an explicit reopen.
- **C12: DA-017 over-read.** `Capability.parallel` is a necessary condition refined by admission.
  The Clarabel and SCIP items were narrowed to an unpublished settings dependence and removed
  from the widening.
- **C13: the DA-030 trigger.** Cold MCP workers are recorded as a candidate "cold-work need"
  trigger pending measurement, not as "none fired".
- **C14: DA-023.** ADR-0224's v2 amendment preceded its implementation, on the sanctioned route.
  The finding is now about status and evidence semantics. The ADR census was corrected to 153
  (91 proposed, 50 accepted, 12 superseded).
- **C15: cause grouping.**
  - Cause 3 keeps its title.
  - DA-042 and DA-043's currency half are not counted under cause 2.
  - DA-044's durability asymmetry is explicitly outside it.
  - The count of restated causes is consistent: two.
- **C16: PS coverage.** PS-03 and PS-G1 are stated as inherited from 2026-10-24 and not
  reassessed.
- **C17: stale numbers.** Corrected: 73 class hints, about 24–25 light recipes of about 215, and
  27k runtime-tooling lines.
- **C18: DA-020 details.** `pse-py` also selects the feature, so under unification the gate is on
  for every build. The marginal effect is the module and the `pse-operations` → `pse-engine`
  edge. I21 is linked by permalink, and Plan 49 recorded DA-020 as "resolved — TOOL05 / I21".
- **C19: Plan 55c wording.** Aligned with its Outcome.

The challenger independently confirmed:
- the ID table and anchors;
- the commit window;
- the locus counts at both baselines;
- the 13 → 19 bindings;
- the defeated gate and §3.2;
- the 48/32 setup literals;
- the three systemd service owners and the supervisor import;
- the uniform agent remedy;
- the settings bounds;
- the span filter and SDK features;
- the per-call currency route and input scope;
- the `_backup` sequence and timeout;
- the vendored-record facts;
- the Clarabel status map.

It did not verify the streams' probe counts:
- 62 of 74 SDK files;
- about 530 SUNDIALS lines;
- 39 of 238 catalogue entries;
- the 27-module cycle;
- the six-pass count.

Those keep their stream attribution.

## How the standard informed this refresh

Each foundation was assessed by the stream named, and reconciled by the coordinator.

**AP-01 — separation of concerns** (S2, S5). The question was which owners change for the
binding scenarios and for Plans 52–55.
- The core and the new inspection and comparison owners localize change well.
- Placement follows generator and host reach (DA-020).
- The supervisor's roughly 10.7k lines hold seven distinct owners behind one legitimate state
  lock (DA-019).

**AP-02 — stable contracts** (S1, S4).
- The solver adapter seam is strong, with exhaustive status maps.
- Published capability is still the weak point (DA-017). Clarabel and SCIP do not publish
  their parallel route's settings dependence, which is minor.
- Agent operations are hand-projected (DA-015).
- Vendored-code contracts are untracked (DA-041).

**AP-03 — composition** (S2, S3).
- The run supervisor, the study policy, inspection and comparison show composition working.
- Per analysis kind, per durable owner and per host, the same questions are answered again
  instead of composed (DA-009, DA-033, DA-015).

**AP-04 — domain model and semantic authority** (S1). Model adequacy and authority were assessed
separately:
- the core model is adequate and governs behavior;
- canonical and boundary declarations describe shapes while closed meaning lives in strings
  (DA-007, DA-040);
- physical meaning and permission are lost at the boundary, now including inspection (DA-005,
  DA-038);
- design intent and vendored provenance have several writable statements per fact (DA-023,
  DA-041).

**AP-05 — explicit structure** (S1, S2).
- Store-enforced attempt transitions are a strength, and so are typed inspection refusals.
- Failure meaning follows the constructor (DA-039).
- A declared feature gate enforces nothing (DA-020).
- Telemetry granularity and loss visibility are conventions (DA-042).
- Workload classes are remembered conventions (DA-036).

**AP-06 — local reasoning** (S2, S5).
- Pure policy cores exist and are consumed.
- The workflow namespace is unchanged (DA-021).
- Tooling owners sit in a 27-module cycle with the production supervisor (DA-037).
- Test-architecture consequences belong to the DTA account.

**AP-07 — execution fit** (S3, S5). Assessed qualitatively for edit/re-solve, studies, dynamics,
concurrency, backup and the agent loop, with H2, H7, H8, H11–H14, H18–H28 most relevant. The
structural gaps are:
- the per-unit publication path and the study population ends (DA-034, DA-035);
- collection and backup verification (DA-025, DA-044);
- per-call agent cost (DA-043);
- declared per-request spans multiplied by the per-unit round trips (DA-034).

Thread pools have one owner per process, with three cross-pool gaps suspected. The per-problem
exclusion was weighed again and left to I01.

**Operational refinements:**
- **Meaning and identity (DP-01–07)** drove causes 1 and 5, DA-023 and DA-041. DP-05–07 were
  only premise-checked; the prefix relationships are unchanged.
- **Transformation and reuse (DP-08–12).** DA-006 and DA-038. DP-09 reuse keys were examined in
  the compiler, where Salsa is sound, and in artifact currency (DA-043). Telemetry is kept out of
  reuse keys, which is a strength.
- **Libraries and ownership (DP-13–17).**
  - Libraries are contained and used natively: FastMCP, Cyclopts, DataFusion, OpenTelemetry,
    Salsa, Cargo freshness and sccache, all checked through Context7 or installed sources.
  - The bespoke duplications are the per-owner protocols, liveness rules, the producer chain and
    the currency key (DA-033, DA-037, DA-024, DA-043).
  - Vendoring discipline is DA-041.
- **Effects, lifecycle and diagnostics (DP-18–21).** DA-034, DA-035, DA-025, DA-044, DA-039 and
  DA-042.
- **Evidence and evolution (DP-22–24).**
  - DA-023 and DA-041.
  - The versioned exchange documents under DA-007.
  - `Wire::FlatV2`, a strength.
  - The MCP wording "retains … units" (DA-005).

**Process-simulator profile** (S4 primary):
- *Assessed in detail, mostly as strengths:*
  - PS-09 and PS-10 across every adapter;
  - PS-07, derivative source per adapter;
  - PS-08, recycle policy, scaling, history and restarts;
  - PS-11, warm starts, IDAS budgets and KLU reuse;
  - PS-12, partial and cancelled outcomes, plus inspection and comparison scope;
  - PS-04 and PS-05, the index-1 check and tear mapping, with the attribution gap in DA-039;
  - PS-06, tear domains.
- *Inherited from 2026-10-24 and not reassessed:*
  - PS-01, except the DA-005 boundary premise check;
  - PS-03, conservation by construction, whose code is unchanged.
- *Gates:* PS-G2 and PS-G3 pass at the adapter seam within the sampled scope. PS-G1 (PS-01–03)
  was not reassessed and keeps its 2026-10-24 judgment.
- *Not assessed:* PS-02 beyond the evaluation-time gate, and PS-13.
- *Leads not raised:*
  - Diffsol event-time sensitivities are finite-difference. That is declared in §13 and recorded
    only as an untyped statistic, and no typed consumer was found. Whether any shooting or fit
    publication claims exactness across non-terminal events would settle it.
  - Tear coordinates get infinite bounds, so restart bound screening is inert. This fails safe
    and is diagnosed.

## Revalidation outcomes

| ID | Outcome at 2026-10-25 | Current scope |
|---|---|---|
| DA-001 | Resolved; residue merged | The typed resolved IR owns syntax; runtime name lookup → [DA-006](semantic-model-and-authority.md#da-006) |
| DA-002 | Resolved; residue merged | Modeling derivations catalogued; runtime re-mints → DA-006; store keys → DA-007 |
| [DA-003](semantic-model-and-authority.md#da-003) | Unchanged | Selected-source inventory and two lexical rules restate roles |
| DA-004 | Resolved | Generated artifact and owner enums |
| [DA-005](semantic-model-and-authority.md#da-005) | Unchanged; premise reinforced | Physical meaning unreadable at the public boundary, including inspection v2; Plan 55c's target names it |
| [DA-006](semantic-model-and-authority.md#da-006) | Unchanged | Runtime formulation re-derives or fills in published classification, identity and roles |
| [DA-007](structure-and-invariants.md#da-007) | Widened | Canonical store meaning outside the registry; inspection v2 keys; evidence `kind` literals now known |
| DA-008 | Resolved | Typed store refusals, effects and worker loss |
| [DA-009](structure-and-invariants.md#da-009) | Unchanged | No owned per-kind operation contract; new surfaces added no arms |
| DA-010 | Resolved | Typed covariance origin, verified fit, PSD external admission |
| DA-011 | Resolved | Generated document catalogue; fingerprint check |
| DA-012 | Resolved; residue merged | One run supervisor; dispatch strand → DA-009 |
| DA-013 | Resolved | One pure prediction policy |
| DA-014 | Closed by decision | Parameter-valued points; other kinds refused (I22, RC12) |
| [DA-015](boundaries-and-composition.md#da-015) | Widened and narrowed | 19 hand-bound agent operations with restated defaults; pse-py inspection and the CLI now derived |
| DA-016 | Narrowed and merged | Residue → [DA-017](boundaries-and-composition.md#da-017) |
| [DA-017](boundaries-and-composition.md#da-017) | Unchanged | Published capability disagrees with admission (KINSOL, PETSc); runtime names backends; minor Clarabel/SCIP note |
| DA-018 | Reframed; closed by decision | Integration patch (I23); size-scope gap → DA-023 |
| [DA-019](boundaries-and-composition.md#da-019) | Unchanged cause; latent manifestation | Deployment contract in tooling; the glob closure omits `scripts/generated/`; telemetry joined by hand |
| [DA-020](boundaries-and-composition.md#da-020) | Widened and reframed | Tooling families, a defeated gate and root-crate telemetry vocabularies in product builds; placement by generator reach (I21) |
| [DA-021](local-reasoning-and-assurance.md#da-021) | Unchanged | Runtime-carrying values; flat workflow namespace |
| [DA-022](execution-fit.md#da-022) | Closed by decision (I16) | Revisit trigger not fired |
| [DA-023](local-reasoning-and-assurance.md#da-023) | Widened | Design-intent authority; in-place ADR-0224 amendment; stacked §24.2 narrative; path sets with three semantics |
| [DA-024](local-reasoning-and-assurance.md#da-024) | Unchanged | Strict producer chain without a consumer; the I24 lane gained telemetry plumbing |
| [DA-025](execution-fit.md#da-025) | Unchanged; consequence widened | Physical collection has no production route; backup re-reads the uncollected inventory |
| DA-026 | Resolved | Per-row locators removed; DataFusion pruning |
| DA-027 | Resolved | Group/re-expand cycle deleted |
| DA-028 | Resolved structurally | Multi-output guarded regions |
| DA-029 | Resolved in the compiler; residue merged | Runtime scans → DA-006 |
| [DA-030](execution-fit.md#da-030) | Closed by decision (I08–I10) | Cold MCP workers are a candidate "cold-work need" trigger, pending DA-043's measurement |
| DA-031 | Resolved; residue merged | Compiler view policy; inline role admission → DA-006 |
| DA-032 | Narrowed and merged | Artifact-profile literals → DA-007 |
| [DA-033](boundaries-and-composition.md#da-033) | Unchanged | Lifecycle envelope re-authored per owner; telemetry reuses existing custody |
| [DA-034](execution-fit.md#da-034) | Unchanged | Publication unit per semantic item, each with the full file protocol |
| [DA-035](execution-fit.md#da-035) | Unchanged | Study population operations as item loops |
| [DA-036](development-tooling.md#da-036) | Literals narrowed; structure unchanged | No resource authority; new 48 = 16 + 32 setup coupling |
| [DA-037](development-tooling.md#da-037) | Widened; footprint corrected | 27-module cycle; three systemd-service owners |
| [DA-038](semantic-model-and-authority.md#da-038) | Unchanged; premise reinforced | Result permission re-derived; inspection reports `Unknown` |
| [DA-039](structure-and-invariants.md#da-039) | Unchanged cause; widened | Failure class by constructor; inspection sites doubled; telemetry and the agent remedy inherit it |
| [DA-040](structure-and-invariants.md#da-040) | New | Boundary declarations stop at field types |
| [DA-041](local-reasoning-and-assurance.md#da-041) | New (narrowed by C6) | Edited vendored trees lack a truthful patch record; prose contradicts the bytes and §13 |
| [DA-042](execution-fit.md#da-042) | New (narrowed by C10) | Prefix-selected export granularity; SDK drop count compiled away |
| [DA-043](development-tooling.md#da-043) | New | Per-call agent lifecycle contrary to LIFE04; currency from a tooling-authored input superset |
| [DA-044](execution-fit.md#da-044) | New | Backup verification under a fixed clock and multi-pass; durability inconsistent with restore, within a declared limit |

Each topic page keeps an anchor and a one-line record for its resolved, merged and closed
identities. DA-001–DA-044 are allocated, retired IDs are never reused, and the next available ID
is DA-045.

## Strengths to preserve

These bound the findings; a later remedy must not discard them. The 2026-10-24 items were
re-verified at `b5bdac9da` where their code changed, and premise-checked where it did not.

- **Physical typing.** One owner holds the complete quantity type
  (`pse-quantity/src/quantity_type.rs`). Side inputs are quantity-keyed Arrow rows checked before
  custody (`workflow/physical_values.rs`).
- **Guards survive rewriting, and derivatives refuse rather than fall back** (`pse-math/src/guarded.rs`;
  `execution.rs::available_order`). No silent finite-difference path was found.
- **The applicability gate is enforced at evaluation.** Unpermitted out-of-envelope inputs raise
  `MathError::Applicability`.
- **Conservation by construction; PSE-S01 is package-only.**
- **Structural admission before solve.** Matching, Dulmage–Mendelsohn and block-triangular
  analysis through pounce-presolve, with refusal by semantic id. An index-1 check runs before
  integration.
- **Truthful solver adapters** (new at 2026-10-25):
  - exhaustive status maps checked against the pinned SCIP 10.0.2, Clarabel 0.11.1, HiGHS, IDAS
    (about 60 flags), PETSc and Uno;
  - limits never mapped to success;
  - certificates only after original-coordinate verification;
  - `quality.rs::qualify` re-grants assurance only from original-space evidence;
  - contained callbacks;
  - analytic derivatives;
  - a private PETSc options database.
- **Library-owned recycles.**
  - A causal sweep is a KINSOL map.
  - Tears come from an exact MILP or an explicit greedy route, with an independent acyclicity
    witness.
  - The convergence policy is in the profile identity.
  - Tear scaling comes from model numerics.
  - History and restarts are bounded.
  - Evaluation uses immutable overlays.
- **Salsa is the in-lane reuse authority** (new at 2026-10-25):
  - complete interned identities (`grouped.rs::BodyInput`);
  - refusals never memoized;
  - setters fire on change;
  - granular declarations;
  - durabilities declared.
- **One pure completion owner.**
  - `workflow/numerics.rs` is consumed by every run-level route except the automatic residual-LM
    fit.
  - Summaries read the persisted decision.
  - Qualification is backend-neutral (`pse-backend-native/src/quality.rs`).
- **One run supervisor and structural request/result pairing** (`RunSupervisor`, `RunOutcome`).
- **One pure study policy**, executed natively and in the store (`pse-model::study_policy`,
  `pse-study-module`).
- **The durable lifecycle's correctness core.**
  - No native work runs inside transactions.
  - Claims are fenced before effects, and study claim custody is per invocation (`80a889ac1`).
  - Terminal admission checks against a frozen acknowledgement inventory.
  - Seed eligibility is sealed with completion.
  - Recovery is per attempt and never re-runs science.
  - Cancellation drains.
  - Attempt transitions are enforced by the store.
  - Refusals carry registry codes.
  - The per-problem creator guard is retained deliberately (I01).
  - Backup copies the complete database and byte-compares every payload file against its
    source. Restore checks the database tree logically through the application inventory. This
    wording was corrected at 2026-10-25.
  - Restore refuses rather than repairs and can resume.
  - Merge is three-way by logical identity, with typed conflicts and a compare-and-set head.
- **Execution core.**
  - Lane-level structural reuse.
  - Compact output shaped by demand, which refuses rather than truncates.
  - Publication lanes that overlap compute.
  - DataFusion-planned result reads with pruning.
  - One in-process CPU owner whose semaphore admits compiler, math and query work.
  - Native teams run at their admitted width.
- **Owned agent-facing contracts** (new at 2026-10-25):
  - native result inspection with bound cursors, byte budgets and typed refusals;
  - evidence comparison with explicit denominators and unmatched or ambiguous subjects, decided
    natively and passed through intact;
  - every new document shape generated once, with `Wire::FlatV2` refusing old headers.
- **Observability is library-native and off the scientific path** (new at 2026-10-25):
  - the OpenTelemetry SDK batch processors, readers, propagators and bridges;
  - Collector queues and retry;
  - OpenObserve PromQL and alerts;
  - explicit host composition;
  - metric names, units and domains declared once in Rust and generated for Python;
  - telemetry kept out of reuse keys.
- **Library containment.**
  - The SurrealDB SDK, Salsa and Symbolica each have one owner.
  - FastMCP (visibility, transforms, middleware, `Client`), systemd, `pyo3-introspection`, Cargo
    `checksum-freshness` and sccache are used natively.
  - The capability CLI is derived from the MCP tool schemas.
  - Tooling reads generated schemas rather than mirroring them.
  - The SurrealDB server patches are an ordered series at an exact revision.
- **No science in Python.** Arrow streams cross at batch granularity, and decoding is strict and
  checked against fingerprints.

## Scope, coverage and limits

**Inspected.** Representative owners and journeys across all product crates and the Python
facade. Every change since `cbf45ccc9` was followed into its dependents. In addition:
- **Prior gaps, now covered:**
  - solver adapter internals;
  - recycle convergence;
  - vendored SurrealDB SDK, engine-API, server, SUNDIALS and Diffsol trees;
  - compiler and Salsa reuse;
  - backup v4 and merge;
  - DataFusion/native thread pools;
  - most of the FastMCP inventory;
  - the supervisor's responsibilities.
- **Tooling side:**
  - the FastMCP capability plane and generated CLI;
  - `pse-env`, host admission and build reuse;
  - the supervisor's deployment, backup and liveness paths;
  - the telemetry services;
  - `xtask` roots;
  - the producer chain;
  - runtime tooling placement.

**Not examined, or sampled only:**
- scientific and numerical correctness of the reference packages;
- PS-02 beyond the evaluation-time gate, and PS-13;
- POUNCE and Ipopt internals beyond their status maps;
- the semantics of the Uno and PETSc patches (DA-018, closed by I23);
- the remaining vendored trees (`suitesparse-sys`, `paste`, `pest_derive`, `py_literal`);
- CPU work inside the SurrealDB server;
- history reads and exchange internals beyond chunking;
- the bodies of the telemetry alert, investigation and selection modules;
- most of `capabilities/specialists.py`;
- `xtask/src/evidence_tools.rs`.

**Unmeasured.** No current elapsed time, file count, span count, call latency, rebuild time,
contention, memory or capacity was measured. The I30 record keeps its original conditions.

**Not proof of absence.** No finding is a statement that unlisted code is aligned.

## Reading and ownership

| Page | Contents |
|---|---|
| [Integrated system account](system-architecture.md) | System picture, functional target (including the agent loop), the five causes, two authority findings, the attractor test, refuted hypotheses, scenarios, improvement directions |
| [Semantic model and authority](semantic-model-and-authority.md) | DA-003, DA-005, DA-006, DA-038 |
| [Structure and invariants](structure-and-invariants.md) | DA-007, DA-009, DA-039, DA-040 |
| [Boundaries and composition](boundaries-and-composition.md) | DA-015, DA-017, DA-019, DA-020, DA-033 |
| [Local reasoning and assurance](local-reasoning-and-assurance.md) | DA-021, DA-023, DA-024, DA-041 |
| [Execution fit](execution-fit.md) | DA-025, DA-034, DA-035, DA-042, DA-044; closed-by-decision DA-022 and DA-030; considered and not raised (thread pools, merge ancestry, inspection decode, the per-problem guard) |
| [Development tooling](development-tooling.md) | DA-036, DA-037, DA-043 |

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
No review consumed the 2026-10-24 refresh at `2257311c0`.

**Refresh rule.** Refresh in place before a new review engagement:
- preserve identities and lineage;
- remove resolved material once source establishes the correction;
- keep the earlier text and its dated evidence in Git and the removal commit.

The 2026-10-24 challenge reconciliation (C1–C16) and the method notes are preserved at
`2257311c0`.
