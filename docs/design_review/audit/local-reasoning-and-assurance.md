# Local reasoning and assurance

These findings cover four questions:
- What must a developer or agent read and instantiate to change a responsibility safely?
- Are effects and ownership visible?
- Do declared contracts and design intent tell the truth about the code?
- Does machinery without a consumer still impose obligations?

The [collection README](README.md) gives the baseline and limits. Citations are `path::symbol`
at `cbf45ccc9`.

## <a id="da-021"></a>DA-021 — Domain values carry the composition root, and the workflow is one flat ownership space

**Principles:** AP-01, AP-06, DP-17, DP-18; G4, G9 (PSE-S05).

**Refresh status:** retained, with a narrowed consequence and widened structure.
- LM eligibility (`fitting/oracle/residual.rs::eligible_inputs`), covariance withholding and the
  study policy now take explicit inputs.
- Plan 49 reports six Runtime-free policy tables, and the test-setup consequence is smaller for
  those policies.

**Diagnosis.**
- **The package value carries the runtime.** `workflow/modeling.rs::ModelingPackage` still holds
  `runtime: Runtime`, and now also `project_context` (a `pse-operations` type) and
  `research_source`. `FitProblem` holds the runtime too, and prepared values return
  `prepared.source.runtime`.
- **The workflow namespace is flat.**
  - `ModelingPackage` has inherent impls in about 36 production files (31 before).
  - `workflow` has 84–98 sibling modules, depending on how they are counted, and about 366
    `pub(in crate::workflow)` items.
  - Plan 51 project merge and preparation code sits inside `workflow/modeling/`
    (`project_comparison.rs`, `project_preparation.rs`).
  - The modeling-canonical ↔ fitting `FitDeclarations` cycle remains.
- **The core's run-level policy is pure; composite drivers are not.** Formulation in
  `cases.rs` and `dynamics.rs` is async and evaluation-coupled (`observe_registered`).
  Horizon move authorization and profile chains are decided inside effectful drivers
  ([DA-038](semantic-model-and-authority.md#da-038)).

**Consequence.**
- **No internal ownership boundary.** Nothing protects one workflow owner's internals from
  another. This is what makes the per-kind reach-ins of
  [DA-009](structure-and-invariants.md#da-009) possible.
- **Every consumer takes the whole runtime closure.** That closure includes the store client,
  DataFusion and Tokio, whether the consumer is product import, an agent host or inspection.
- **Some policy is testable only through the full stack.** Changing advanced-step move
  authorization, or how formulation classifies targets, means reading a large async driver and
  running closed-loop native tests.
- **A better pattern already exists.** `pse-study-module` depends only on `pse-model` because the
  study policy was extracted.

**Counterevidence.**
- AP-01 says crate splitting is not modularity.
- §3.2 and ADR-0175 deliberately make the runtime the owner of scientific workflow.
- Sampled runtime science consumes compiler and backend products rather than restating them.
  The exceptions are [DA-006](semantic-model-and-authority.md#da-006) and DA-038.
- The run-level completion policy is pure.

**Evidence:** *Implemented* for fields and counts. Consumer cost is reasoned.

**Improvement direction.**
- Narrow internal interfaces per workflow owner: kinds, durability, projects, results,
  formulation.
- Explicit policy inputs supplied by the composition root.
- Pure decision functions for composite drivers.

## <a id="da-023"></a>DA-023 — Design intent has more than one writable statement per fact, and decision status no longer says what is in force

**Principles:** AP-04, AP-06, DP-01, DP-22, DP-24; G1 (applied to declared intent).

**Refresh status:** retained and grown. Plan 49 claimed resolution (TOOL07, QUAL05). Source
contradicts that claim for every example the prior account named.

**Diagnosis.**
- **The sections claim authority, but their labels mislead.**
  - The reading guide (§0.1) makes the sections the sole contract owner. It also says a page
    "describes implemented behavior unless it labels a statement *Proposed*".
  - The sections carry 268 `> Decision` blocks (239 at the previous baseline), 31 supplements and
    2 amendments, plus about 405 plan references. 174 lines cite a *proposed* ADR for
    implemented behavior.
  - "Proposed" is thereby overloaded between ADR status and the evidence label.
- **ADR status no longer tracks what is in force.**
  - 90 of 152 ADRs are `proposed`, against 50 accepted; no ADR has been accepted since the
    previous baseline.
  - Proposed records stay editable, and the supersession machinery engages only for accepted
    ones.
  - Thirty proposed ADRs carry `evidence: Proposed` ("nothing is built"), although several
    describe running contracts.
- **ADR-0164 and ADR-0214 contradict each other.**
  - ADR-0164 (proposed, not superseded) still states that application RPC moves to a WebSocket
    transport preserving RocksDB.
  - ADR-0214 (proposed, `supersedes: []`) selects SurrealMX over gRPC, which is what the code
    uses.
  - §20 paraphrases both.
- **Amendments stack instead of replacing.**
  - In §7.1, an ADR-0215 block announces an immutable resolved semantic product, while the
    constraint table beneath it still says "no intermediate IR is produced or stored".
  - Named drift persists verbatim:
    - `PreparedSimulation::rebind` (§13.5; no such type);
    - `crates/pse-runtime/src/generated/` (§4.2; absent);
    - Delta `CHECK`s (§14);
    - `crates/pse-operations-queries/`, listed as a generated path in AGENTS.md,
      `.claude/rules/generated.md`, `.claude/settings.json`, `scripts/agent-hooks.py` and
      `scripts/semantic_edits.py`, though retired at `06302af77`.
- **Implemented-but-unused routes are presented as implemented.** The native child route
  (§18, `numerical-execution.md`) is described as "the implemented child route" without saying
  that it has no production caller ([DA-024](#da-024)). Uno SQP's supported problem size appears
  only in proposed ADR-0157 ([DA-018](boundaries-and-composition.md#da-018)).
- **Retirement is blocked.** Completed plans 28–53 are retained "for qualification and proposed
  ADR readers" (`docs/plans/README.md`). The ADR lifecycle therefore blocks ADR-0096 retirement.

**Consequence.**
- *PSE-S06 (upgrade the SurrealDB SDK or transport):* an agent following the binding's "where
  roles are decided" meets contradictory, editable ADR outcomes and `evidence: Proposed` on
  running contracts.
- *Change a contract:* immutability protects only 50 records, so proposed rationale can be
  edited in place and silently diverge from the sections.
- Readers cannot tell what is in force from what is aspirational without reading code.
- The same ambiguity reaches closure. Plan 49 marked all 32 prior findings resolved. Source
  shows 13 continuing as active findings and 2 more merged into them, and four closure claims
  contradicted by unchanged files (see the README's revalidation table).

**Counterevidence.**
- §24.4 records a deliberate local record lifecycle: maintainer-authorized implementation
  without PR-route acceptance, for a single maintainer.
- AGENTS.md says ADR acceptance is not implementation acceptance, and sections own contracts.
- As governance, that survives. As an authority design, it does not:
  - the sections still depend on ADR status words and plan status;
  - proposed records still carry consequential outcome text that contradicts successors.

**Evidence:** *Interface-checked*. The counts come from section text and front matter. The
coordinator read the ADR-0164/0214 contradiction and the §7.1 self-contradiction directly.
Whether other proposed ADRs contradict sections was sampled, not surveyed.

**Improvement direction.**
- One writable statement per design fact.
- Section text states current contracts without packet, plan or qualification status.
- ADR status and evidence truthfully indicate what is in force, with replacement links whatever
  the status.
- Retained plans are not load-bearing for decision records.
- Generated-path and protected-path sets are derived from one declaration.

## <a id="da-024"></a>DA-024 — Bespoke machinery runs without a production consumer

**Principles:** DP-16, H14, H20, H26, H27; AP-07.

**Refresh status:** narrowed and widened.
- `pse.canon.v2` was removed.
- The native child lane is retained as bounded exploration by maintainer decision I24. Its parent
  side is still constructed only by tests, and the child ships as `xtask` `pse-native-lane`.
- The strict producer-provenance chain is newly identified.

**Diagnosis.** A strict producer-qualification chain exists. It includes:
- `xtask/src/producer_identity.rs` (7.5k lines), `producer_preparation.rs` and
  `producer_history.rs`;
- `scripts/producer_deployment.py` and `producer_cargo.py`;
- about 1.9k lines of hand-reviewed `.config/producer-{build,macro}-reviews.json`;
- `pse-buildinfo` identity;
- startup paths in `pse-runtime/src/math/portable.rs`.

The chain yields a `QualifiedProducer` and `ReplayAdmission`, stored in
`workflow/canonical.rs::CanonicalDeployment::producer`. That accessor has one caller, a test
(`study_tests.rs`). Ordinary preparation stopped using canonical portable reuse at `401f0d70a`.
Qualification tooling still validates the chain's own receipts (`scripts/validation.py`
producer gates).

**Consequence.**
- Under routine dependency bumps, which AGENTS.md sanctions, a reviewed build-script or macro
  crate stops matching its review entry, and the strict route refuses persistent reuse. Nothing
  uses that reuse, so the cost is pure maintenance (H27).
- The startup qualification work runs at every Python or worker deployment open.
- The guides describe a reuse capability the product does not exercise.

**Counterevidence.**
- Public portable APIs remain valid caller-composed capabilities.
- The outer build attestation is genuinely consumed for result lineage, and these paths share it.
- DP-16 permits a bounded exploration purpose. None is declared for the strict producer route,
  although one is declared for the native child.

**Evidence:** *Implemented* for the absence of production consumers, by search across `crates/`,
`xtask/` and `python/`. Whether an unrecorded maintainer intent keeps it is open.

**Improvement direction.** Producer qualification exists only with a consumer or a declared,
bounded exploration purpose, and its startup and maintenance cost follows that purpose. The
contract text states which routes are exploratory.

## Moved identities

[DA-022](execution-fit.md#da-022), hand-maintained size models, is closed by maintainer decision I16.
Its record and revisit trigger are on the execution-fit page.
