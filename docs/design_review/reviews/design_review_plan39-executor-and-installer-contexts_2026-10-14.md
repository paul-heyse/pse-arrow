---
title: Plan39 executor and installer context boundaries
date: 2026-10-14
tier: change
purpose: target
status: review
standard: Core 3.4
heuristics: Efficient Architecture 1.0
profile: Process Simulator 1.5
baseline: 0c72f3403
---

# Plan39 executor and installer context boundaries

**Decision: Accept the target at Proposed strength.** Source-qualified executor premises and explicit launcher/installation phases are an appropriate design for one strict preparation operation. The target retains actual first-compilation evidence, complete relevant inputs, immutable original build provenance and independently current installed/import context. It admits only a qualified exact installation transition; it does not claim that a changed original namespace remains current. This is architecture acceptance, not qualification of a projected PSE closure or an actual installation.

This is a bounded independent assessment by `/root/executor_context_design`. It reviews the maintainer-selected target in the uncommitted “Concrete context decision required before D1” section of [Plan39d](../../plans/39d-qualification-and-plan38-cutover.md) and the uncommitted “Source-qualified compiler, launcher and installation contexts” amendment to proposed ADR-0189 over `0c72f3403`, with concurrent formatting changes in `producer_preparation.rs`. It does not accept that implementation or qualify PSE. Plan39d D0 owns disposition; Plan39/ADR-0189 own the decision route. The earlier [preparation-operation review](design_review_plan39-preparation-operation_2026-10-14.md) remains historical evidence for its narrower target.

## 1. Scope and strongest evidence

The functional outcome is one strict preparation operation that composes actual runtime, worker and Python builds with actual installation/import. Preserve selective scientific producer assurance, source-qualified external-I/O completeness, exact original and installed artifact association, conservative unknown handling and meaningful current deployment validation. Ordinary Local admission, scientific models, solver adequacy, service behavior and whole-product qualification are outside this review.

Core 3.4, Efficient Architecture 1.0 and the pse-arrow binding govern this assessment. Process Simulator 1.5 is selected, but PS-G1–PS-G3 do not apply: this change neither changes nor establishes physical formulation, well-posedness or numerical outcomes. Relevant drivers are AP-01–AP-07, DP-08/DP-09/DP-19 and H12/H13/H18–H20: reuse facts within their validity scope and separate effects that change that scope.

**Interface-checked:** read `producer_identity::{native_caller_environment,native_review_basis,reviewed_input_state,expand_reviews,executable_closure_basis,add_dep_info,deployment_association}`, `producer_preparation::{executor_bases,verify_compilation,Capture::enter,Capture::validate}`, `DeploymentAssociation::verify_current_scoped`, the build/macro review models and selected PyO3 catalog entries, plus `producer_cargo`, `producer_deployment` and `python_extension` composition. Also read the relevant blueprint §5.3 owner and proposed ADR-0189.

Pinned local sources examined: PyO3 0.29.2 `pyo3-build-config::{resolve_build_config,from_pyo3_config_file_env,from_cargo_dep_env}`; Maturin 1.15.0 interpreter discovery, develop/install backend and editable metadata paths; uv 0.12.22 wheel uninstall. Maturin/uv source copies were supplied under `/tmp/pse39-final-execution/python-context-investigation/`. Source review does not independently establish that a particular actual invocation reaches every examined branch.

No tests, builds, services or measurements ran for this review. Existing targeted test claims were not rerun or adopted as new evidence. The full native/tool/helper closure was not audited for projection completeness.

## 2–5. Responsibilities and revealing scenarios

Four different facts must retain their owners:

| Owner | Fact established | Required boundary |
|---|---|---|
| Executor preparation | A selected executable was actually compiled, and this operation can reuse its exact outputs | Complete relevant compiler/source/helper/configuration/native inputs and actual first compilation; current executor-owned premises |
| Source-qualified launcher | The actual interpreter/startup/helpers selected the captured child context | Complete applicable I/O/branch review, actual arguments/configuration/environment and immutable evidence of that invocation |
| Installer | This exact admitted distribution caused only the permitted changes | Selected existing distribution/removal inputs, exact install payload/tool context, authenticated before/after transition |
| Deployment verifier | The current installed role and its required current inputs agree with the admitted result | Exact imported/mapped artifact, current final namespace/context and explicitly selected phase semantics |

**S01 — Maturin adds PyO3 and venv selections to an unrelated executor.** `executor_bases` currently hashes the root role's native context and caller environment into every executor key. Removing the role's review assertion in v2 is useful but leaves this coupling. `verify_compilation` independently rechecks the preceding whole-role association. A source-qualified executor view must govern both the key and its current-premise verification; merely changing the key cannot make its evidence relevant. Existing source-bound owner I/O grants cover executable I/O, not compiler environment projection by implication.

**S02 — The same executor source compiles under changed environment consumption.** Rust `env!`/`option_env!`, generated suppliers, cfg/features and upstream `rustc-env` outputs can change executable bytes without changing package identity. Actual associated dep-info retains compiler environment dependencies, but a prior dep-info hash alone does not show that their current effective values match. Cargo-injected values also differ from the capture process's ambient values. A sound projection treats compilation and executable I/O separately, includes the whole executable dependency closure and refuses an unresolved channel.

**S03 — Explicit PyO3 config avoids discovery, while Maturin still launches Python.** PyO3's exact `resolve_build_config` returns from the explicit config branch before host/cross discovery; that branch opens raw absolute-path config bytes and applies build environment. This supports a narrower reviewed Rust-build closure. Maturin's examined interpreter discovery runs Python with `-c` and no site-disabling option. A PyO3 branch exclusion does not remove the launcher's startup inputs or authorize their later mutation.

**S04 — Installation mutates a launcher-consumed namespace.** `verify_current_scoped` compares original namespaces on each raw/installed association. `bind_python_artifact` invokes that check again before association. Maturin's selected uv install path uses `install --no-deps --force-reinstall`, followed by an editable metadata adjustment. uv uninstall reads the selected distribution's RECORD, removes entries and can remove directories and `__pycache__` outside the literal RECORD file list. A distribution-directory wildcard or “ignore metadata” policy would not describe these effects soundly. Conversely, retaining the old whole-site digest as a final unchanged premise refuses legitimate installation.

**S05 — An unexpected concurrent or startup-induced mutation accompanies installation.** Before/after equality of the non-output portion must still detect it. A successful installer exit and a new digest do not explain a change. Existing operation exclusion covers cooperating writers, not arbitrary tools. Launcher startup can execute `.pth` code: complete membership/bytes do not alone establish that its effects are complete. Its source-qualified I/O review and original before/after checks remain necessary.

## 6. Assessment

These verdicts concern the amended target, including its explicit complete-input, transition and final-verification requirements. They do not assert that the current implementation realizes it.

| Foundation | Verdict | Reason |
|---|---|---|
| AP-01 | Satisfied at Proposed strength | Compiler, launcher, installer and final verification have distinct reasons for change. |
| AP-02 | Satisfied at Proposed strength | Final verification explicitly consumes unchanged build inputs, historical launcher evidence, a qualified transition and current installed context. |
| AP-03 | Satisfied at Proposed strength | One operation composes existing Cargo, Maturin and uv responsibilities without role-local cleaning. |
| AP-04 | Satisfied at Proposed strength | Executor premises, immutable build/launcher context, permitted installation change and current installed context have distinct governing meanings. |
| AP-05 | Satisfied at Proposed strength | Unknown projection, unsupported branch and unexplained installation changes refuse; an explicit phase relationship replaces accidental script ordering. |
| AP-06 | Satisfied at Proposed strength | Small compiler and installation fixtures can test these boundaries without a canonical store or simulator. |
| AP-07 | Satisfied at Proposed strength | Relevant executor reuse removes incidental invalidation; the phase chain supports legitimate installation without repeated destructive refresh or fictitious namespace equality. No speedup is measured. |

G1–G3 **pass at Proposed strength**: each phase has an explicit authority and applicable input/transition contract; old and current states retain their different meanings. G4 **passes at Proposed strength** because the target explicitly exposes launcher/install effects. G5/G6 **pass at Proposed strength** with complete transition reconciliation, terminal outcomes, actual first compilation and same-operation reuse under unchanged relevant premises. G7 **passes at Proposed strength** because uncertainty and campaign limits are explicit. G8 **passes at Proposed strength**: Cargo still compiles, Maturin packages and uv installs; a source-qualified observation/association adapter is justified, while reimplementing their general selection algorithms is not. G9 **passes at Proposed strength**, following the foundation judgments. PS-G1–PS-G3 are **not applicable** for the scope stated above.

## 7. Finding and correction

<a id="f01"></a>
### F01 — Root-role input context is the wrong relevance scope for shared executor compilation

**Cause — Interface-checked.** `executor_bases` adds the entire root-role native context and `native_caller_environment` to every selected executor. `verify_compilation` also verifies the preceding whole-role association. A change known only to the root installer can therefore invalidate an unrelated executable even when its dependency-complete source and actual unit context agree. Equal unit keys alone remain insufficient evidence, so simply ignoring the difference would be unsound.

**Consequence.** S01 makes ordinary cross-role sharing depend on unrelated root SDK selections, while S02 shows why an unqualified environment/file projection would lose compilation dependencies. AP-01/AP-04/AP-07, DP-09 and G6/G9 require the executor's actual complete scope, rather than either global overbinding or a guessed exclusion.

**Correction — Proposed.** The selected target introduces one current source-qualified executor view for both the reuse basis and original-premise verification. It explicitly covers compiler dependencies and executable I/O, mandatory tool/configuration/native contracts and exact source/helper/features/platform. Unknown/stale/incomplete projection refuses. Original executor outputs remain exact and current; whole-role final deployment validation retains its own phase contract. The field/validation requirements below make the source-qualification obligation concrete without mandating another registry.

**Verification.** In a real compiler fixture, a proven unrelated executor can reuse its prior actual compilation after supported Maturin-only changes. Consuming `env!`/`option_env!`, generated/absent-to-present channels and changed source/helper/compiler/native premises refuse. Prior actual dep-info is independently associated; effective Cargo-owned values are handled as such. A missing/stale projection remains conservative. Actual PSE source completeness is a separate qualification obligation.

<a id="f02"></a>
### F02 — One unchanged namespace assertion spans an installation effect that changes it

**Cause — Interface-checked.** Current `verify_current_scoped` checks the original consumed namespace, and `bind_python_artifact` applies it again after installation. The examined Maturin/uv path legitimately replaces site entries. A protected historical copy does not make those original paths current, and a new digest alone does not explain the change.

**Consequence.** In S04, the old verifier refuses a supported install. Rebinding its namespace or dropping its verification can instead admit an unexplained changed input and misstate historical evidence as current. AP-02/AP-04/AP-05/AP-07, DP-08/DP-09 and G1–G3/G5/G6/G9 require an explicit relationship between original provenance, permitted effects and current state.

**Correction — Proposed.** The selected amended target defines a final phase association/verifier consuming immutable original build/launcher evidence, an independently checked exact installation transition and a current installed-context observation. It verifies unchanged executor/build inputs at original paths; treats the pre-install launcher snapshot as historical; verifies every admitted namespace change through the transition; and verifies the final installed/import context as current. It does not call the old complete pre-install association “current.” Unsupported or unexplained transitions refuse. This intentionally changes the original final-current realization through RC02, now selected by the maintainer.

The smallest sufficient contract need not introduce a new registry or service. Existing declaration/evidence owners can carry these meanings. The following information is necessary, whatever the physical representation:

- **Executor projection review:** contract/version; exact owner and executable closure qualification; selected unit/features/platform; qualified compiler/configuration/native contract references; complete applicable compile-time and executable-I/O channels, including relevant absences and latent generated selectors; explicit supported branches. Unknown/stale qualification grants no exclusion. Conservative fallback still validates actual compiler environment/generated dependencies or refuses; the old finite caller-environment tuple and whole-role file verifier alone do not establish that guarantee.
- **Executor premises:** operation/unit key; projection-review binding; actual successful compilation evidence and associated dep-info for the dependency closure; effective compiler-environment dependencies with their Cargo/ambient/generated origin accounted for; exact original outputs; current file, namespace, environment and absence observations. One admitted view supplies both identity and verification. Do not retain the preceding role's full association as an unexplained executor dependency or simply delete its protection.
- **Launcher association:** exact tool/interpreter/startup/helper and source-qualified I/O context; original namespace observations; actual selected child argv/cwd/configuration and relevant environment; input consistency outcome and immutable raw evidence. Arbitrary `.pth` execution is subject to I/O completeness, not qualified by a namespace snapshot alone.
- **Installation transition:** original association and before-state binding; exact wheel/distribution payload; qualified installer/tool/options; exact selected existing metadata/RECORD and removal scope; permitted output derivation, including source-established generated metadata, directory and bytecode effects; complete after-state and raw reconciliation. An observed delta, path prefix or self-reported RECORD must not authorize arbitrary changes. Changed bytes, modes, links, membership and absence premises all require reconciliation.
- **Final admission:** explicit guarantee/version; immutable predecessor/transition linkage; complete applicable current installed/interpreter startup context and exact imported artifact; existing full-byte RPATH replay evidence; unchanged non-output premises; terminal success. New startup/import channels require source-qualified completeness, not merely a post-install namespace snapshot. Failure/interruption cannot publish a complete phase chain.

**Validation order.** Establish current review/branch applicability and operation ownership first. Verify a preceding executor's complete projected original premises before giving Cargo effects. After success, associate actual compiler outputs/dep-info, verify effective inputs and before/after consistency, then enroll first compilation or same-operation reuse. Seal original build/launcher evidence before install. Bind the exact selected distribution/removal inputs before installation, reconcile its full effects after it, independently import/replay/check the installed artifact, then validate and publish the final phase association. Any evidence arriving too late to authorize a prior effect cannot retroactively grant that effect; unsupported sources/branches remain conservative.

**Verification.** Tests must distinguish an allowed transformation from renamed omission: config-file removal cannot inherit the no-discovery branch; exact permitted install delta admitted while an unexplained sibling file, `.pth`, import-shadow candidate, link or mode mutation refuses; changed old RECORD cannot expand authority; interrupted install cannot publish; protected old evidence stays reconstructible without being called current. Actual PSE closure qualification and all-role install/import remain Plan39d acceptance, not fixture claims.

## 8–10. Alternatives, library fit and evidence limits

Retaining the current full context is the simplest safe fallback, but leaves strict D1 unqualified when legitimate installation changes a consumed namespace. Per-role cleaning, arbitrary Fresh reuse, receipt rebinding and namespace omission do not serve the selected guarantees. Substituting Local would be an explicit different scope decision.

The recommended phase chain preserves exact historical provenance and independently current installed context while allowing only a source-qualified installation transformation. Its cost is a bounded versioned association and explicit source-review obligation for the reached installation behavior. A broad generic installer model would be disproportionate; unsupported branches should refuse. Reusing actual Cargo/Maturin/uv outputs and extending the existing observation owners is preferable to reproducing their package-selection logic. Dynamic metadata requires a source-backed rule and authenticated inputs, not fixed guessed filenames or a directory-wide exception.

The baseline diagnoses and PyO3 branch behavior are **Interface-checked**. The selected executor/phase-chain remedy is **Proposed**, not implemented, tested or measured. Missing concrete PSE projection and installer reviews are future implementation/qualification obligations; their absence alone is not proof that the design is impossible. They prevent implementation eligibility and actual D1 acceptance until established, without preventing acceptance of the explicit fail-closed target architecture.

## 11. Rule impacts and disposition

<a id="rc01"></a>
**RC01 — Executor-owned relevant premises replace root-role premises for finite compilation reuse.** Clarify proposed ADR-0189 and blueprint §5.3/§14.3 so an explicit source-qualified projection can supply complete relevant executor inputs and original-premise verification. Every role retains its own native eligibility and actual artifact association. Retaining the current aggregate rule means Maturin-specific context can continue to invalidate unrelated executors; no arbitrary exclusion is granted. This impact belongs to the already selected narrower per-executor correction route; its concrete proof remains required.

<a id="rc02"></a>
**RC02 — Final current admission distinguishes immutable prebuild evidence from authenticated installation changes and current installed state.** The old requirement is recorded in blueprint §5.3, ADR-0189's finite-operation amendment and the prior preparation-operation review, whose acceptance expressly excluded namespace pruning. The selected replacement is the explicit phase-chain guarantee in F02, now recorded in the proposed ADR-0189 amendment and Plan39d. It retains exact provenance and current final validation but intentionally changes the statement that every original consumed namespace remains unchanged after installation. The maintainer explicitly selected this target on 2026-10-14; formal ADR status, enduring blueprint amendment and implementation acceptance remain separate. If the old rule were kept, changed consumed namespaces would refuse and D1 would remain unqualified unless another conforming design were selected.

Plan39d D0 is the single disposition owner for F01/F02 and both impacts. Proposed ADR-0189 now records the selected semantic changes; the coordinator owns the remaining enduring blueprint route and implementation disposition. Formal ADR status and actual assembled acceptance remain separate. No SHOULD exception or MUST waiver is proposed.

## 12. Decision

Behavioral adequacy and architectural fitness **pass at Proposed strength** for the amended target with its explicit complete-input and phase-chain contracts. Overall **Accept the target**. F01/F02 remain implementation findings owned by Plan39d D0; accepting their correction does not resolve them.

The next obligation is to realize one admitted executor view and the selected historical/transition/current verifier semantics, then obtain the actual source-qualified PSE and installer contracts before granting eligibility. Unresolved source projections or branches must continue to refuse. Actual all-role capture/install/import determines whether those concrete contracts support the intended operation. Native completeness, scientific/store/recovery journeys and comprehensive qualification remain unexamined here and owned by Plan39d. No numerical performance benefit is claimed.
