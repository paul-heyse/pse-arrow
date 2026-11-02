---
title: "Coding-agent implementation brief"
notion_source: "https://app.notion.com/p/40b141b583698179a1c4c44393fc70f3"
notion_last_edited: "2026-11-01T23:34:00.115Z"
---

# Coding-agent implementation brief

Source: [Notion](https://app.notion.com/p/40b141b583698179a1c4c44393fc70f3). Imported 2026-11-02.

[Plan detailing and industry coverage assessment](../rationale/plan-detailing-assessment.md)

**Purpose:** supplement the published implementation plan with decisions and bounded delivery briefs that a capable coding agent can execute autonomously. **Baseline:** `pse-arrow` commit `19e7b49a67df24c1bf73005a6186e145e930d53a`, confirmed unchanged for this review. This is adopted implementation guidance, incorporated on 2026-11-01 into [Implementation sequence and first delivery](implementation-sequence.md). No builds, numerical tests or repository implementation changes were performed by the documentation update.

These adopted briefs retain P0–P7 and their existing scope and evidence owners. Packets A–D detail P1 without recording implementation progress or introducing an additional gate or broad audit. The applicable basis is Core 3.4, its efficiency guidance, ProcessSimulator 1.5 and review guidance, the repository binding, and test-assessment 1.2. [Selected standard](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/standard.toml).

## I01 — What the packet should decide, and what the agent should decide

The current six-chapter plan establishes the scientific ambition and architecture. Each implementation packet should now identify the observable behavior it must deliver, the existing owner that may change, the settled scientific choices and the evidence that could disprove the implementation. Supply those facts and delegate local engineering judgment.

| Fix in the accepted packet | Delegate within that boundary |
| --- | --- |
| Engineering problem, reference revision, physical assumptions, selected parameter records, units/bases/datums, branch and approximation meaning | Internal module/function organization and implementation sequence |
| Existing source, identity, preparation, execution and result authorities; permitted effects | Reuse of existing helpers, suitably scoped new helpers, batching and economical preparation |
| Public behavior: inputs, outputs, supported edits, expected refusals and source attribution | Exact names and internal Rust types unless they cross an established durable/public contract |
| Required scientific products and comparison allowances | Smallest discriminating tests and diagnostics that exercise those products |
| Native versus external-reference boundary and intentional local versus recorded execution | Suitable existing native recipes and resource profiles under repository policy |
| Explicit consequential decisions still unresolved | Ordinary package extensions and local repairs that do not alter those decisions |

Do not require user approval for each internal interface or new dependency. The repository already defines when an ADR is required, including changes to its named architectural decisions, adding/removing crates or major dependency-family changes. A capable agent should follow that route when applicable and otherwise proceed. Its brief should distinguish unresolved scientific meaning from implementation freedom. [Repository instructions](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/AGENTS.md); [design-change workflow](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/design-change-workflow.md).

## I02 — Make the domain-to-code binding operational

Use existing public seams, not conceptual domain names as instructions to create matching classes:

- **Source creation/editing:** `ModelingPackage` and source services own immutable declarations and physical context. `with_declarations` computes additions, replacements and removals by declaration ID. It explicitly does not retain old document text as the authority of edited IR. Therefore choose the editing representation deliberately: text edits use the existing source/document route; declaration edits create an admitted declaration revision. Do not promise simultaneous authoritative text and object representations or invent a synchronization layer. Preserve source lineage and any supported rendering separately.
- **Selected topology:** `prepare_flow` takes `ModelingFlowSelection` and uses semantic preparation independently of numerical incidence. It is not an instruction to compile the full mathematical problem for every display request. Source-level inspection can report declarations; instantiated inspection must state its selected configuration and resolved scope. Neither is a persistent second flowsheet graph.
- **Case and procedure:** `declared_execution_with_demand` admits demanded value paths before later numeric rebinding. `prepare_declared` accepts the Solve procedure only. `execute_declared` executes Solve/Initialize, but turns an incomplete initialization into an error. The planned failure-restoration journey should use `initialize_declared` when it needs the full `ModelingInitializationReport`, rather than losing its stage/attempt evidence behind an “execute” wrapper. Private `resolve_case` is an implementation owner, not a new frontend API to expose automatically.
- **Local studies and recording:** `admit_study` retains the exact immutable package/physical owners without contacting a canonical store; `LocalStudy::execute` uses those owners. `record_study` is the explicit publication route. A frontend should compose these choices, not serialize `LocalStudy`—which deliberately has no serialization contract—or add an implicit store fallback.

These are concrete source constraints the coding agent should preserve. Exact convenience APIs remain delegated until their consumers are clear. [Source and topology APIs](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/modeling.rs); [declared procedures](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/modeling/declared.rs); [study ownership](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/study.rs).

## I02/I03 — Invariants expressed as meaningful change scenarios

Use the following to select acceptance checks; they are not a new mandatory test framework or recurring audit.

| Change scenario | Invariant and implementation consequence |
| --- | --- |
| Rename an equipment label | Preserve its authored identity and existing references. Renaming a contractual state-slot role is a different semantic edit; `StateKey` includes the role and actual index values, not a display ordinal. |
| Reorder component presentation | Connections, outputs and comparisons remain keyed by semantic component identity. Missing or unlike component/transport coverage requires explicit translation. |
| Edit feed temperature | Rebind an already admitted numerical demand; retain its physical type and source. A newly demanded path/alias is not merely another numeric value. |
| Change rating into design | Alter fixed/free roles through the case owner. A changed free-variable layout is a structural case change, not a value-only edit. |
| Replace a property-data record | Re-admit actual dependencies and applicability. Reuse cannot rely on a property-package display name or stale coefficient copy. |
| Apply a source edit incrementally | The admitted meaning and selected results must agree with clean reconstruction; old immutable revisions remain interpretable. Do not require byte-identical incidental numerical ordering. |
| Fail during initialization | Preserve the original specification and actual stage/attempt outcomes; temporary fixes or simplified laws cannot become the accepted operating model. |
| Stop before a candidate exists | Return unavailable assessments and original failure/limit meaning through the selected operation. Empty result data is not physical infeasibility, successful simulation or a zero-valued answer; recording remains explicit. |

Current state/connection and runtime case tests already exercise several of these responsibilities. Reuse their scientific expectations where the same responsibility is under test; add only missing consumer coverage or a distinct wrong-case discriminator. [Process contracts](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-modeling/src/specialize/process.rs); [existing contract tests](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-modeling/src/process_contract_tests.rs); [case resolution](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/modeling/cases.rs).

Keep assurance roles distinct: admission/policy checks should require only their real inputs; native integration checks must exercise the consumed native path. Shared domain types and valid setup are useful, but expected calculations must challenge the specific production mistake under test. An expected failure must reach the intended boundary and fail for the intended reason; a fixture/setup failure cannot satisfy it. This needs no universal test framework or second implementation for every assertion. [Test-assessment §§1–8](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/core/test-assessment.md).

## I01/I08 — An extension-locality decision procedure

When an industrial case does not fit, first state the missing scientific relationship or operation in one paragraph. Then locate its smallest existing owner:

1. **Different coefficients, species, phase sets, geometry or constitutive equations:** extend authored data/definitions and their actual dependencies.
2. **A different state convention:** compose or extend state coordinates, reconstruction and transport observations. Check whether the apparent gap is only the seed `ThermoPackage` interface; do not promote a seed limitation into a kernel restriction.
3. **A missing scientific distinction:** show why existing physical types, subjects, contribution roles or declaration forms cannot express it without losing meaning. Extend the owning generic contract once, with a small independent witness and the real consumer.
4. **A numerical limitation:** identify the exact derivative, DAE, discrete, event or certification requirement against available routes. Inspect fitting native-library capabilities before extending their integration owner; library-owned iteration, globalization and factors remain there. Add bounded scientific composition only for a demonstrated consumed-contract gap, rather than implementing a new solver loop.
5. **A missing view or report:** project existing facts at their owner. A public boundary may need a typed document, but a new screen or equipment name does not justify a new relation family.

The agent need not produce throwaway prototypes when source already settles the question. It should stop a proposed shortcut when it would hard-code species/equipment names in generic Rust, infer scientific validity from solver success, erase an unsupported physics distinction, or duplicate authoritative records. Surface the concrete unresolved choice to the coordinator and continue independent work; do not launch a broad architecture audit. [Model composition](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/models-and-composition.md); [schema authority](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/schema-and-relations.md).

For consequential physical organization, preserve semantic owners without forcing one object, query, task or native call per unit. Compare complete inspection/edit/re-solve work: sufficient selected input, stable preparation, compact views, live workspaces and explicit publication. Prefer the simpler conforming route; no new scheduler, cache or performance campaign follows without a concrete need. [AP-07 and efficiency guidance](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/core/efficient-architecture-heuristics.md).

## I01 — Four concrete next delivery packets

Each packet belongs in the existing owning plan, with its outcome, baseline, allowed changes, settled/open decisions, dependencies and evidence. Do not add a separate agent ledger, readiness ontology or approval checklist.

### Packet A — Ordinary authoring inputs for the first two process models

**Deliver:** ordinary admitted package/case inputs using the existing recycle and exchanger definitions, plus the separate original conformance/reference cases. Fix the intended editing representation and parameter provenance. Preserve exact thermal, material, smoothing and costing assumptions. Repair only the owning scope descriptions needed to adopt this expanded target.

**Evidence:** inspect admitted source/physical identities and selected dependencies; demonstrate that ordinary inputs do not acquire oracle-only eligibility. Keep the exchanger's unknown-area and pressure-extrapolation permissions explicit: matching costing arithmetic is not a validated purchase-price estimate.

**Stop condition:** missing ordinary scientific records, unclear datum conversion or incompatible reference assumptions. Resolve that scientific input; never relabel oracle data or weaken admission merely to make the demonstration run. [Existing recycle](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/campaign/models/recycle-flash.pse); [existing exchanger](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/campaign/models/exchanger-costing.pse).

### Packet B — One usable process inspection/edit path

**Deliver:** inspect unit occurrences, resolved ports/state contracts, connection occurrences, selected material/property configuration and case roles; perform identity-preserving rename, operating-value edit and one topology edit through current source/case owners. Expose selected-scope diagnostics and exact revision attribution. A full graphical editor is unnecessary.

**Evidence:** old/new revision correspondence, component-order independence, invalid connection attribution and incremental-versus-clean preparation for those edits. Use existing semantic preparation only where instantiated information requires it. Preserve parallel connections and selected isolated units.

**Stop condition:** the proposed view needs independently mutable physics, treats numerical array positions as identity, or silently changes public case semantics. Adjust the projection/owner rather than creating another object graph.

### Packet C — Execute and inspect the complete first journeys

**Deliver:** default recycle initialization/solve, its intentional initialization failure, and the exchanger design solve through native operations. Include the same exchanger's rating case only with an explicit revised fixed/free specification. Return physical outputs, original checks and typed outcomes; preserve full initialization reports for the failure journey. Exercise explicit local execution and the selected retained-result path.

**Evidence:** role-dependent degrees of freedom and structural admission before solving, original residual/bounds/domain checks and independent component/energy closure, restoration after failure, and exact semantic output readback. Structural admission does not establish uniqueness or global physical validity. Reuse immutable established checks only while their premises hold; each new numerical state still needs its applicable post-solve checks. A changed feed value must not inherit stale science from reused preparation. Expected diagnostic failures can pass their assertions without producing usable numerical candidates. [PS-04, PS-10 and PS-13](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/profiles/process-simulator/principles.md).

**Stop condition:** success requires a hidden retry, stage values become model facts, or unavailable assessment is presented as failed physics. Repair the existing outcome/procedure integration; do not mask it in the frontend.

### Packet D — Matched independent reference and P1 closeout

**Deliver:** isolated live IDAES counterparts for the selected positive cases, complete observable correspondence and the relevant failure/diagnostic assertions. Preserve analytic recycle and hand-derived exchanger checks. Extend existing native exchanges and conformance products; do not replace them with notebook-exit or golden-file-only assertions.

**Evidence:** pinned inputs/environment, finite complete mapped outputs, declared physical allowances, wrong-value/missing-output controls, original physical acceptance and complete selected population accounting. Select P1 integration/reference and scope-end checks under repository workflow once these changes are integrated.

**Stop condition:** reference prerequisites are unavailable, case conventions differ, or outputs cannot be compared. A selected required live-reference invocation with missing prerequisites fails; genuinely unrun scope remains unattempted. Neither is a successful skip, and neither permits broadened tolerance or removal from the denominator. Existing zero-failure qualification obligations remain; broader Plan53/58h and 200/1,000-case campaign obligations retain their owners. [Current parity command](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/justfile); [conformance owner](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/crates/pse-runtime/src/workflow/modeling/conformance.rs).

## I08 — Keep the architecture open to industrial breadth

Before freezing broader public authoring contracts, examine representative cases that challenge different semantics. Execute the additional scientific witnesses when their selected family enters scope; they are not automatic new P1 qualification obligations. A useful existing example is the custom liquid–liquid extractor: distinct single-liquid material systems, unequal component memberships, volumetric/mass-concentration coordinates and explicit solute transfer. It tests whether the implementation has accidentally equated “process model” with the seed's molar two-phase state. It does not establish electrolytes or general extraction accuracy. Its notebook also contains differing unit presentations across example/test blocks, so resolve the executable reference specification before treating printed values as a golden target. [Pinned custom-unit tutorial](https://github.com/IDAES/examples/blob/eb64e38ac429b7e3ae4f8db071f8d8cad2eef675/idaes_examples/notebooks/docs/unit_models/custom_unit_models/creating_unit_model.ipynb).

Further selected witnesses should stress multiphase inventories, distributed transfer, nonconventional material characterization, scheduled operating modes and joint experiments as those industrial requirements enter scope. The coding consequence is consistent: industry-specific science belongs in packages; reusable semantic gaps belong at existing generic owners. Do not create an `Industry` enum, universal plant superclass or preemptive capability/plugin framework.

## Delegation and remaining uncertainty

Parallelize independent scientific packages or reference implementations after shared physical contracts settle. One coordinator integrates changes to shared state/port/schema/operation contracts; independent reference work should not derive its expected values from candidate production code. Keep briefs and checkpoints at the existing plan owner.

During implementation, follow repository-targeted checks and generated-source ownership; regenerate when an owning generator or registry declaration changes, move all callers and remove genuinely replaced mechanisms in the same change. Reserve broader qualification for the selected integrated scope. The agent may decide routine implementation details but must not turn a source-level observation into Tested/Measured evidence. Current external example compatibility, ordinary-data eligibility, runtime results and performance remain to be established by execution. This guidance fixes the work's meaning and acceptance; it does not pre-approve a specific implementation or claim those uncertainties are resolved.
