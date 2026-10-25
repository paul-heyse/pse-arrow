# Local reasoning and assurance

These findings cover four questions:
- What must a developer or agent read and instantiate to change a responsibility safely?
- Are effects and ownership visible?
- Do declared contracts and design intent tell the truth about the code?
- Does machinery without a consumer still impose obligations?

The [collection README](README.md) gives the baseline and limits. Citations are `path::symbol`
at `b5bdac9da`. Each *2026-10-25 refresh* note says what changed or was re-read since `cbf45ccc9`.

**Evidence dates.** Earlier *refresh* paragraphs retain the input account's source and scope;
*followup* notes and the [current revalidation table](README.md#revalidation-outcomes) identify
this engagement's re-readings and corrections. Earlier exact counts and measurements were not
renewed by republication.

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

## <a id="da-023"></a>DA-023 — Current-contract text retains obsolete mechanisms and conflicting descriptions

**Principles:** AP-06, DP-22, DP-24; G1 only where two current statements give incompatible
meaning, rather than merely recording different evidence or decision states.

**2026-10-25 followup at `682fc3830`: narrowed and corrected.** The previous account combined
real current-text drift with three insufficient grounds for a violation:
- A `proposed` ADR may describe *Tested* behavior. Formal decision acceptance and implementation
  evidence are deliberately separate under the local lifecycle. Their coexistence is not a
  contradiction, and the count of proposed ADRs establishes no alignment gap.
- ADR-0214 explicitly says "This supplements ADR-0164". Blueprint §20 places its selected
  MX/gRPC contract immediately after the older substrate decision. The absence of `supersedes`
  metadata does not establish two active transport authorities. Navigating historical rationale
  may be awkward; the current transport is identifiable.
- The three path consumers do different work. `agent-hooks.py::protected` enforces agent edit
  policy, `semantic_edits.py::owned_path` admits a deliberately narrower saved-edit capability,
  and `workspace_context.py::symbol_context` reports an `edit_owner` hint. Different path answers
  are not, by themselves, inconsistent enforcement of one rule.

The proposed-record lifecycle, decision-PR route and deliberate retention of qualification
owners are not rejected by this finding. Previous counts and broad claims about decision status
are withdrawn as diagnostic grounds; their exact input is preserved at `c02f68670`.

**Diagnosis retained.** Some text presented as a current architectural contract has not been
reconciled with its replacement:
- Blueprint §7.1 introduces the immutable resolved semantic product, while the constraint table
  still says `pse-compiler::typed_math` lowers the authored AST and "no intermediate IR is
  produced or stored". `pse-modeling::resolved` is the implemented product. The surrounding
  distinction from a competing arithmetic interpreter does not make the AST-only lowering
  statement current.
- Blueprint §13.5 directs readers to `PreparedSimulation::rebind`; that type is absent from
  the current Rust source. Rebinding exists on actual modeling simulation owners.
- Blueprint §24.4 restates role effort selections that differ from their declared owner,
  `.agents/roles/README.md`. The architecture says Opus/medium for implementation review and
  Opus/high for design review; the role owner says high and xhigh respectively.
- Retired generated paths remain in instructions and protection lists. This is stale navigation,
  not evidence that protecting a nonexistent path is itself an unsafe implementation.

**Consequence.** A developer following the documented extension or preparation route must
resolve incompatible or nonexistent entrypoints against source before making an otherwise local
change. This is consequential where it misidentifies the semantic lowering owner; the stale path
and effort examples are supporting, bounded documentation defects. They do not justify a general
claim that the scientific implementation has competing authorities.

**Counterevidence.** The reading guide names architecture sections as contract owners, ADRs as
rationale and source inspection as the way to settle implementation. The transport supplement
and separate acceptance/evidence states work as declared. Specification and implementation are
legitimate separate descriptions; divergence calls for reconciliation, not removal of either.

**Evidence:** *Interface-checked*. The coordinator re-read §7.1, §13.5, §20, §24.4, ADR-0214's
Context, ADR-0224's front matter, the role owner, all three path consumers, and searched current
Rust source for the named obsolete type. No architecture or governance files were changed.

**Improvement direction.** State each current mechanism at its owning section, replace obsolete
entrypoint descriptions when contracts change, and reference role/policy owners for mutable
configuration. Keep historical rationale, formal decision status and implementation evidence
separate and explicit. Correction is established when the documented semantic lowering and
rebind routes resolve to the implemented owners without contradictory current instructions.

**Related:** [DA-041](#da-041) concerns false records of dependency source bytes, a distinct
provenance problem; it is not evidence that proposed decision status is invalid.

## <a id="da-024"></a>DA-024 — Portable producer/replay machinery is optional; ordinary startup and its consumer scope must be distinguished

**Principles considered:** DP-09, DP-16, AP-07; H14, H20, H26, H27.
**Followup disposition at `682fc3830`: previous broad maintenance/startup diagnosis not established.**

The previous account conflated three things:
- consumed outer executable/build attestation;
- optional strict producer-receipt qualification for public caller-composed portable APIs;
- the absence of ordinary in-repository persistent replay consumers.

`CanonicalDeployment::open_observed` always observes the deployment. In
`math/portable.rs::observe_deployment_runtime`, strict `QualifiedProducer` receipt loading/verification happens
only for `Some(path)`; `None` returns no producer while retaining the outer attestation.
`xtask/src/bin/support/deployment.rs` selects the receipt from `PSE_PRODUCER_RECEIPT`, and Python
configuration also makes the strict receipt optional. Without a strict receipt, `open_observed`
separately attempts `observe_local_runtime` to mint local receiving admission. That is distinct
from strict receipt qualification, and its marginal role/cost needs its own assessment.
Strict receipt qualification therefore does not run
unconditionally at every Python/worker open.

The accessor `CanonicalDeployment::producer` has only a test caller in the searched
`crates/`, `xtask/` and `python/` trees. Ordinary preparation no longer consumes canonical
portable replay. That bounds the current default route; it does not establish that the public
portable capability lacks an architectural role. DP-16 permits an architectural role or bounded
exploration without requiring an in-repository production caller. The native-child lane also
retains its explicit maintainer-approved exploration purpose (I24).

**Unresolved execution question.** Attribute any avoidable ordinary-startup work to the exact
unused replay admission or observation step, separately from mandatory consumed outer lineage
and optional receipt validation, before calling it excess machinery. No marginal cost or
unnecessary full strict-chain execution was demonstrated in this followup. The former line-count
and dependency-review maintenance inventory does not settle that question.

**Improvement direction, conditional.** Describe default, public caller-composed and exploratory
routes truthfully. If a specific unused default-startup step is shown to add material work,
localize it to the capability that consumes it while preserving outer result lineage. No deletion
of the portable capability, its tests or consumed provenance is prescribed by this account.

**Evidence:** *Interface-checked*. The fixed-baseline accessor search, `open_observed`,
`math/portable.rs::observe_deployment_runtime` and the xtask/Python receipt selection were read; no startup benchmark
or functional portable-reuse campaign ran. Keep this identity as a corrected scope record, not a
demonstrated open violation based solely on missing in-repository consumers.

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
4. **No complete regeneration patch is retained; the current SDK has substantive and whitespace divergence.**
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
- Retain truthful, complete source/edit information sufficient to preserve intended behavior
  when re-vendoring or rebasing. The existing patch-series form is a candidate; a complete
  inventory or reproducible diff may also satisfy the need. The current tree can be diffed;
  whitespace drift makes that noisy, not impossible.
- Reconcile false "untouched" prose and identify upstream-defect workarounds through the existing
  R-41 route where its contract applies.
- Assess an explicit OTel family only if its type-sharing upgrade obligations warrant it. The
  absence of a family row does not itself establish a current resolved-version split.
- No new proof apparatus or universal patch format is selected by this audit. Correction is
  demonstrated by a truthful record and a re-vendoring path that preserves the declared edits.

**Related:** [DA-023](#da-023) (design-intent authority), [DA-018](boundaries-and-composition.md#da-018)
(the declared Uno precedent), [DA-017](boundaries-and-composition.md#da-017).

## Moved identities

[DA-022](execution-fit.md#da-022), hand-maintained size models, is closed by maintainer decision I16.
Its record and revisit trigger are on the execution-fit page.
