# Local reasoning and assurance

These findings cover four questions:
- What must a developer or agent read and instantiate to change a responsibility safely?
- Are effects and ownership visible?
- Do declared contracts and design intent tell the truth about the code?
- Does machinery without a consumer still impose obligations?

The [collection README](README.md) gives the baseline and limits. Citations are `path::symbol`
at `b5bdac9da`. Each *2026-10-25 refresh* note says what changed or was re-read since `cbf45ccc9`.

## <a id="da-021"></a>DA-021 — Domain values carry the composition root, and the workflow is one flat ownership space

**Principles:** AP-01, AP-06, DP-17, DP-18; G4, G9 (PSE-S05).

**Refresh status:** retained, with a narrowed consequence and widened structure.
- LM eligibility (`fitting/oracle/residual.rs::eligible_inputs`), covariance withholding and the
  study policy now take explicit inputs.
- Plan 49 reports six Runtime-free policy tables, and the test-setup consequence is smaller for
  those policies.
- **2026-10-25: unchanged.** S2 counted the same structure at both baselines:
  - 143 non-test workflow files;
  - 98 top-level workflow modules;
  - 36 `impl ModelingPackage` files;
  - 33 `runtime: Runtime` fields;
  - `pub(in crate::workflow)` items 366 → 368.

  Result inspection arrived as an `impl Runtime` method, with no new runtime-carrying values.

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

**2026-10-25 refresh: widened.**
- **ADR counts.** Of 153 ADRs, 91 are proposed, 50 accepted and 12 superseded (90 proposed and
  50 accepted before). The Diagnosis below keeps its 2026-10-24 figures.
- **ADR-0224** is still `proposed`, with front matter `evidence: Tested`.
  - Its v2 "Plan55 nested inspection amendment" (`e41abcdc8`) came *before* the v2
    implementation (`0031e9ff2` onward). Editing a proposed record is the sanctioned route, and
    Plan 55's rule decisions record it.
  - The issue is the status and evidence semantics: a proposed record carrying a Tested outcome
    for running behavior.
- **ADR-0225** carries `evidence: Tested`, while its Confirmation says "Proposed" and a Tested
  note is appended below. §23.1 depends on the status word ("ADR-0225 remains a proposed
  decision record").
- **§24.2** gained about 60 lines of dated per-plan qualification narrative linking into the
  retained Plan 52h and 55f outcomes. It says it "does not replace the historical full-product
  basis", so the amendments stack.
- **Retained plans.** Completed Plans 52 and 55 are retained "for qualification/proposed-ADR
  readers" (`docs/plans/README.md`), which extends the retirement block.
- **Named drift persists verbatim:**
  - the ADR-0164 / ADR-0214 contradiction;
  - §7.1 "no intermediate IR";
  - `PreparedSimulation::rebind`;
  - the retired `crates/pse-operations-queries/`, still listed in AGENTS.md, `.claude/settings.json`,
    `.claude/rules/generated.md`, `.claude/path-allowlist.txt`, `scripts/agent-hooks.py` and
    `scripts/semantic_edits.py`.
- **The protected-path implementations now differ in meaning:**
  - `semantic_edits` protects all of `docs/adr/` and any path part named `generated`;
  - `agent-hooks` protects only non-proposed ADRs and `crates/*/…/generated`;
  - `workspace_context` uses `"/generated/" in relative`.
- **Further drift, older than this refresh.** `sections/design-change-workflow.md` states model
  allocations that disagree with `.agents/roles/README.md`: design review on Opus/high and
  implementation review on Opus/medium, against opus/xhigh and opus/high.
- **Counterexample.** The §21.1 amendment replaced its old paragraph rather than stacking on it.
- **A record-provenance analogue in vendored code** is [DA-041](#da-041).

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
- **2026-10-25: unchanged.**
  - The producer-chain files have no diff in the window.
  - `CanonicalDeployment::producer` still has one caller, a test.
  - `open_observed` still runs startup observation at every deployment open.
  - The native child lane that I24 retains as bounded exploration gained Plan 52 telemetry
    plumbing:
    - the W3C carrier and settings propagation in `math/native_child.rs`;
    - `initialize_private_child_host` in `pse-native-lane.rs`;
    - a paragraph of §24 lifetime semantics.

    Its parent is still constructed only by tests. This is maintenance growth on an exploratory
    route, which I24 governs.

**Diagnosis.** A strict producer-qualification chain exists. It includes:
- `xtask/src/producer_identity.rs` (7.5k lines), `producer_preparation.rs` and
  `producer_history.rs` (about 10k lines together);
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

## <a id="da-041"></a>DA-041 — Edited vendored trees have no truthful patch record; their prose records contradict the bytes and the architecture

**Principles:** DP-01, DP-15, DP-16, DP-22, DP-24, AP-02; G1 (applied to provenance) (PSE-S06
and the matching solver-library bump).

**Refresh status:** new at 2026-10-25. It covers an area the previous account did not examine.
Most of its manifestations predate `cbf45ccc9`; Plan 52 repeated the pattern. It is a bounded
authority finding beside [DA-023](#da-023), with a distinct mechanism and footprint. Challenge
C6 narrowed it before publication: the draft's headline, "an undeclared numerical SUNDIALS
hunk", is withdrawn (see below).

**Diagnosis.** The repository vendors code in two ways.
- **Patch series, which are well governed.** Patch files against a pinned upstream with an
  archive sha256:
  - Uno and PETSc through `[workspace.metadata.pse.{uno,petsc}].patch`;
  - the SurrealDB server through `scripts/surreal_source.py::PATCHES` at an exact revision.
- **Edited vendored trees with a prose record.** `vendor/surrealdb`, `vendor/surrealdb-engine-api`
  and `vendor/sundials-sys`, described by `vendor/native-supplier-notes.md` and
  `vendor/surrealdb/PATCH.md`. Nothing ties the prose to the bytes.

Every manifestation below is in the second mechanism.
1. **SUNDIALS C source is patched, and its record says it is not.**
   - Twelve files differ from the checksum-verified `sundials-sys` 0.6.2 archive. They include
     `idas.c`, `idaa.c`, `idas_ic.c`, `idas_impl.h`, `idas_io.c`, `sunlinsol_klu.{h,c}` and
     `wrapper.h`, about 530 changed lines.
   - They landed between `d85408066` and `a791c533e`, from 2026-10-17 to 2026-10-21.
   - The notes file was last edited at `478c99f6f` (2026-10-14). It says that every packaged file
     "matches that archive except the local edits described here", and that "All upstream feature
     selections and numerical Rust/C sources are preserved". It describes only build-script
     corrections.
   - That contradicts the bytes. It also contradicts §13 (`workflows-and-results.md`), which
     states that "the bundled IDAS owner refreshes primal and sensitivity quadrature derivative
     history" after consistent initialization.
   - There is no hunk-level record of the changes. The new symbols (`IDAInitialQuadDerivatives`,
     `IDASetCheckpointLimit`, `KLUFixedPattern`, `IDASetAcceptedStepBudget`) appear in no
     `vendor/*.md` record.
   - The quadrature fix is also absent from register R-41, the existing record of upstream-defect
     workarounds, which carries a "pinned release fixes it" trigger.
   - The vendored `diffsol`, `diffsol-la` and `diffsol-nl` trees, which the same notes cover,
     differ only in `build.rs` and `Cargo.toml` (S4). The notes' claim holds for them.
2. **The SDK record is incomplete, and a second record is stale.**
   - `vendor/surrealdb/PATCH.md`, last edited 2026-10-21, omits Plan 52's change `78b26a9d9`:
     - the OpenTelemetry dependencies;
     - W3C `traceparent` injection in `engine/remote/grpc_lifecycle.rs`;
     - a test-support feature.
   - `vendor/patches.md` still describes "packaged 3.3.0 … revision 238bfeb" with WebSocket
     controls. Production selects only `protocol-grpc`.
3. **Other records lag.**
   - `scripts/surrealdb-source/source-build.md` gives reasons for 8 of the patch inputs. It
     omits the five `surrealism-*` patches that `PATCHES` applies.
   - `.config/producer-build-reviews.json` still keys its SDK grant to `vendor/surrealdb#3.3.0`.
4. **The vendored SDK cannot be regenerated as a patch.**
   - 62 of the 74 upstream `src/*.rs` files differ in bytes, and 16 of those differ only in
     whitespace, spaces against upstream tabs.
   - The substantive delta is about +2.8k/−0.5k lines across 50 SDK files, plus about
     +1.2k/−0.15k in the engine API.
   - `rustfmt.toml` ignores `vendor` to "keep upstream source bytes", which the tree already
     does not do.
   - The vendored manifest also declares exact OTel pins (`=0.33.1`/`=0.34.0`).
     `[workspace.metadata.pse.families]` declares no OTel family; it lists only arrow,
     datafusion, object_store and pyo3. So `just family-check` cannot detect an OTel version
     split across the workspace and the vendored SDK.

**Classification, integration first.** Almost all hunks are integration, consistent with the
maintainer's view that vendored hooks are diagnostics, accounting or admission integrations:
- SDK: request context, bounded admission, response-envelope preflight, no-replay restoration
  and W3C context.
- Server: session binding and caps, status trailers, durable-profile refusal, Surrealism caps
  and deadlines, and MX controls. SSI is a native MX option declared by ADR-0214.
- SUNDIALS: IDAS accepted-step, checkpoint and replay budgets, and KLU work counters.
- SUNDIALS KLU fixed-pattern symbolic reuse is an opt-in factor-reuse lifecycle. Generic mode
  keeps upstream semantics.

One hunk changes numerical behaviour in library core: `IDAInitialQuadDerivatives` refreshes
quadrature derivative histories after `IDACalcIC` (`idas_ic.c`; commit `8dc1f9a9e`). It is
declared and protected elsewhere:
- §13 states the behaviour;
- §24.2 records Plan 45f qualification of "corrected IDAS quadrature histories";
- the same commit added `pse-backend-native/src/dynamics/idas_initial_tests.rs`, compiled through
  `mod idas_initial_tests;`. It contains
  `quadrature_histories_use_corrected_algebraic_initial_state_and_sensitivities`, a scheduled
  variant and a callback-failure test.

So the draft's "undeclared numerical hunk that nothing would catch" was wrong. What is missing is
the vendoring record, not the declaration of the behaviour.

**Consequence.**
- *Upgrade (PSE-S06 and a SUNDIALS bump).* Re-vendoring from the records would drop the
  API-adding hunks, and bindgen or compilation would fail loudly. The quadrature behaviour would
  be lost too, but the existing tests would catch it. In either case the records give the upgrader
  no hunk inventory to re-apply. The architecture and the vendoring record make opposite claims
  about the same bytes.
- *Rebase.* A SurrealDB 3.3.x rebase meets an incomplete PATCH.md, a stale second record and a
  whitespace-diverged tree.
- *Dependency family.* An undeclared OTel family can resolve two context versions after a partial
  bump. This affects telemetry only.

**Counterevidence.**
- ADR-0189 (proposed) and §3.1 permit exact-version vendored sources, retaining "minimal patched
  dependency source with upstream version/provenance". §3.1 states that "local source
  provenance … remain separate obligations".
- ADR-0189's "Withdrawn environment/provenance proof targets" section (2026-10-14) deliberately
  eliminated provenance *proof* work, with no revisit trigger. That decision legitimately
  excludes a proof or check apparatus. It does not make a false prose record true.
- The commit messages describe each change.
- Dropping an API-adding hunk fails loudly.
- The architecture describes the W3C behaviour, so only the rebase record lacks it.
- The patch-series form, used for the server and native-solver patches, shows the repository
  already has a truthful mechanism.

**Evidence:**
- *Implemented* for the SUNDIALS, SDK and engine-API deltas. S4 downloaded or extracted the
  pinned archives into scratch, verified them against the recorded sha256 and VCS revision, and
  diffed them against the vendored trees.
- The coordinator re-read the SUNDIALS commit history, the notes file's last edit and claims, and
  the `IDAInitialQuadDerivatives` call sites.
- The challenger confirmed §13, the tests and the ADR-0189 withdrawal.
- Settling the rest needs two things:
  - diffs of the remaining vendored trees (`suitesparse-sys`, `paste`, `pest_derive`,
    `py_literal`), which are suspected and unexamined;
  - a check of whether a later SUNDIALS release includes the quadrature fix. That is R-41's
    trigger.

**Improvement direction.** These use mechanisms the repository already has; no new check or
proof apparatus is implied (ADR-0189's withdrawal stands):
- Edited vendored trees move to the existing patch-series form against a verified archive
  (`[workspace.metadata.pse.{uno,petsc}].patch`, `surreal_source.PATCHES`).
- Upstream-defect workarounds are entered in R-41.
- Prose records stop asserting "untouched".
- The OTel crate set becomes a declared family.

**Related:** [DA-023](#da-023) (design-intent authority), [DA-018](boundaries-and-composition.md#da-018)
(the declared Uno precedent), [DA-017](boundaries-and-composition.md#da-017).

## Moved identities

[DA-022](execution-fit.md#da-022), hand-maintained size models, is closed by maintainer decision I16.
Its record and revisit trigger are on the execution-fit page.
