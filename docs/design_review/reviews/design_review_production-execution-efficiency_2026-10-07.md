# Production execution efficiency

**Date:** 2026-10-07  
**Tier / purpose:** DESIGN / TARGET  
**Boundary:** Production execution from authored source admission through compilation, preparation, native numerical execution, canonical publication, reopened results and studies, including the build and native installation paths supporting those operations.  
**Standard:** Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and the [pse-arrow binding](../design_principles/binding/pse-arrow.md).  
**Baseline:** Inspection began on `main` at `219338742915fb16c02534c80430bd2ea97bf94a` plus the frozen Plan 28 continuation. During publication, commit `5260a3e9a3cecd69b6358ab86d4e917ae2508b29` recorded that continuation and the supporting evidence. A before/after content comparison confirmed unchanged production, configuration and test bytes. This review is not release qualification.  
**Method:** Fresh-context independent design-reviewer (Sol/high) assessment of authoritative contracts, current source and version-backed supporting evidence. The reviewer did not implement the subject or run product checks. The coordinator inspected decisive source evidence, reconciled library-transfer corrections and publishes this judgment.  
**Disposition owner:** [Plan 28 coordinator](../../plans/28-surrealdb-unified-substrate.md#finding-dispositions). [28e](../../plans/28e-rebuild-retirement-and-qualification.md) continues to own qualification, measurements and E5 acceptance. This document is a dated review, not another status ledger.

## Assessment

**Architectural fitness: Revise. Behavioral and scientific adequacy: interface-checked for the examined contracts; assembled scientific adequacy remains unresolved. Overall decision: Revise.**

The current architecture has substantial strengths worth retaining. Physically typed authored definitions govern scientific meaning. Selected dependency discovery replaces ordinary whole-source export. Immutable mathematics, evaluator artifacts and attempt-owned numerical state have distinct owners. Native libraries supply iteration and factorization. Canonical operations preserve protected source selection, fenced ingestion, complete result admission and truthful partial outcomes. These are meaningful improvements over the older preparation and publication paths.

The strongest remaining defects concern work imposed by physical realization:

- Numerical coordinate projection searches the complete resolved-target vector separately for every variable and row.
- Scalar-index admission splits solve-variable results into six-row IPC blocks and solve-constraint results into eight-row blocks, followed by sequential append operations.
- Successful append acknowledgments return the complete payload, while block reads establish metadata and payload through two sequential protected RPCs.
- Durable study preparation reconstructs selected source and fresh generic compiler admission for each ready occurrence, even when its source and structural premises are unchanged.

These diagnoses follow inspected production source. They do not depend on assigning the observed test durations to a particular stage. Corrections should remove repeated lookup, framing, transfer and admission work before changing numerical algorithms or increasing concurrency.

Native installation verification and outer build attestation expose additional material amplification. Their remedies require explicit lifecycle and provenance decisions: removing checks without replacing their guarantees would be unsound. Narrow math-library opportunities also exist, particularly structural Taylor zeros and reusable numeric factor storage, but the present evidence supports conditional proposals rather than claims that a missing optimization explains current timings.

The review does not recommend reducing production accuracy, weakening original-space assessment, changing the selected server resource profile, increasing test concurrency or tuning timeouts. Contextual engineering accuracy remains a required input to the design.

## 1. Target, scope and coverage

The functional target is a process simulator supporting physically meaningful square simulation, optimization, dynamics and estimation, with interactive edit/re-solve, studies, recycles and extension. Speed matters across the complete operation, including build/setup, source acquisition, structural preparation, numerical work, publication and reopening. A fast kernel cannot compensate for repeated surrounding work.

The relevant variation axes are:

- A value-only case change versus a structural or physical-context change.
- A new unit, property or reaction model using existing concepts.
- A replacement solver, evaluator or sparse-linear integration.
- More variables, rows, study occurrences, retained revisions or trajectory samples.
- Selective versus broad result demand.
- Concurrent claims, cancellation, interrupted acknowledgment and restart.
- A local build edit affecting a scientific producer versus unrelated outer source.

The deployment premise is the selected supervised local SurrealDB service and its existing finite server/worker allocation, including the selected 32 GiB total application allocation: 16 GiB for the server and two 8 GiB workers. No distributed availability or capacity SLA is invented. Existing finite case, workspace, transport and native admission bounds remain relevant; they are safeguards, not evidence that every admitted operation has efficient physical execution.

Inspected source includes compiler workspace/modeling admission and artifact keys; runtime canonical source selection, physical admission caching, study preparation/dispatch and numerical policy consumers; mathematical evaluator/implicit differentiation paths; native quality and dynamics factor construction; result projection, IPC framing, canonical append/read functions and their generator; native preparation scripts; and outer build identity generation. Supporting exact-release evidence covers the relevant SurrealDB, DataFusion, Symbolica, Numerica, faer, Salsa and native integration contracts.

The authoritative owners examined include blueprint §5, §7, §8, §14, §15–§19 and §20, with current Plan 28 context and ADR-0163/0164. These describe the subject; the standard and functional target govern the judgment.

This is not a new scientific qualification, Plan 28 completion audit, E5 acceptance or production implementation assignment. No new benchmarks, EXPLAIN probes, native runs or configuration changes were made. Unexamined scientific families are not declared defective merely because this review did not qualify them.

## 2. Responsibilities and dependencies

| Responsibility / owner | Hidden decision and consumed contract | State/effects and dependency direction | Local reasoning and test boundary |
|---|---|---|---|
| Authored physical/scientific declarations; `pse-modeling`, quantity owners | Quantity meaning, applicability, selected bindings, contributions and original obligations | Scientific declarations govern downstream preparation; storage does not choose physics | Model admission can be examined independently of numerical iteration |
| Selected source acquisition; runtime `workflow/modeling/canonical.rs` | Sufficient lexical/import/member/supplier closure, including negative inventories | Canonical operations provide protected exact revision reads; runtime owns scientific discovery | Requires canonical acquisition evidence for storage-dependent closure |
| Compiler workspace and mathematical preparation | Checked selection, specialization, structure/value distinction, complete dependencies | Pure tracked preparation consumes immutable inputs; native compilation is outside tracked queries | Fresh compilation is the comparison oracle for reuse |
| Mathematical service and library adapters | Artifact construction, bounded immutable retention and attempt-owned workers | Symbolica/Numerica own arithmetic and derivatives; faer owns sparse factors | Library boundaries require actual shape, order, failure and ownership contracts |
| Numerical policy and original assessment | Contextual scales, physical budgets, normalized transport and candidate permission | One resolved policy feeds normalization, native controls and original checks | Policy projections should consume compact admitted coordinates |
| Native adapters | Class-specific iteration, model/session management, status mapping and cancellation | Native mutable state remains owned by the actual numerical execution; no store transaction spans solving | Adapter tests require the selected native capability, not unrelated workflow initialization |
| Durable workflow/study owner | Occurrence identity, dependencies, starts, claims and effect recovery | Composes immutable preparations with canonical state and attempt execution | Pure occurrence policy is distinct from storage and native effects |
| Result projection and canonical execution | Exact authoritative IPC plus derived indexes; fencing and complete visibility | Runtime derives scientific indexes; generated store functions perform structural writes | Codec, projection and lifecycle behavior have separate verification questions |
| Reopened result/analysis owner | Exact selected run/attempt, protected reads, physical interpretation and completion | Selects metadata before payload; Arrow/DataFusion serve admitted analytics | Representation replacement belongs here, without reinterpreting scientific outcomes |
| Build/native installation owner | Actual executable association and admitted native capability closure | Tooling owns preparation and provenance; scientific product keys consume relevant dependencies | Build and installation observations are separate from solve measurements |

The composition root is identifiable in Runtime and its numerical/durable workflow owners. A responsibility boundary need not require a fresh workspace, small IPC block or separate RPC. The defects below concern those physical choices, rather than a need to merge semantic owners.

## 3. Meaning, contracts and preservation constraints

The examined model captures consequential distinctions: physical quantity versus raw coordinate; point versus difference; mass versus molar basis; reference state; fixed/free/parameter roles; topology versus incidence versus solve order; selected implicit function versus residual relation; original problem versus numerical representation; candidate versus qualified result; occurrence versus equal binding; semantic reuse versus buffer retention versus durable history.

These distinctions govern construction and execution in the inspected paths, rather than existing only in result schemas. The review establishes no new AP-04 defect within that examined scope. Broad scientific fidelity still requires the owning qualification evidence.

### Physical semantics

| Quantity or model element | Dimension/unit | Basis and convention | Validity | Authority and consumed operation |
|---|---|---|---|---|
| Temperature, pressure and other physical coordinates | Complete quantity type and canonical unit | Point/difference and explicit datum, including gauge/absolute distinctions | Checked physical compatibility; positive finite normalization where required | Blueprint §8; quantity admission before mathematical construction |
| Material flows and thermodynamic responses | Declared physical units | Species/element subject and mass/molar basis; response reference state retained | Provider/model applicability and domain envelope | Blueprint §9 and authored scientific bindings |
| Balance contributions and closure | Each contribution’s declared physical unit | Conservation sign and accumulation meaning | Original closure budgets remain distinct from mathematical feasibility | Blueprint §10 and workflow assessment |
| Numerical coordinates and physical tolerances | Coordinate scales and budgets projected in admitted order | Frozen contextual characteristic scale; integer lattice preserved | Missing/invalid budgets refuse; trial values do not invent scale | Blueprint §16; resolved policy, normalization and native quality |
| Implicit functions and derivatives | Typed selected function and ordered derivative axes | Explicit branch/sheet and derivative convention | Guards, regularity, bounds and selection evidence | Blueprint §7; guarded preparation and implicit derivative owners |
| Retained result rows and trajectories | Original units, identities and sample coordinates | Exact stored row/field coordinates and declared analysis scope | Partial, unavailable and cancelled states remain explicit | Blueprint §19–§20; completion and canonical read owners |

A physical acceptance budget is not an output-error certificate. Feasibility, stationarity, objective quality, derivative response, closure and engineering output accuracy retain their separate meanings. Improvements must preserve that separation.

### Well-posedness

Variable roles are declared in the case/model contracts. Compiler structural analysis retains equality rows, free columns, incidence and semantic identities. Class-aware native admission consumes that analysis before solving: roots require complete square equality matching; NLP admission permits genuine optimization degrees of freedom while diagnosing unmatched equalities. Topology, incidence and execution partitions are distinct. Matching does not establish numerical rank.

The relevant source and contracts are blueprint §15, `PreparedCase.structure`, compiler structural preparation and native structural/routing owners. An indexed policy projection or reused checked source must not bypass these conditions.

### Required guarantees

All remedies preserve:

- Complete selected dependencies, including absence, membership, imports, interpretation and provider context.
- Original-domain guards and admitted branch/derivative meaning.
- Exact scientific payload bits and declared row/sample multiplicity and order.
- Contextual engineering scales and original-space residual, bound, closure and candidate checks.
- Recorded starts and reuse dependencies.
- Attempt fencing, ordinal coverage, complete admission and truthful partiality.
- Protection through required source/result consumption and durable root admission.
- Definite-conflict retry versus uncertain-acknowledgment settlement.
- Resource charges until actual native work and escaped buffers drain.

## 4. Revealing scenarios

| ID | Stimulus / kind | Expected boundary | Current observation and evidence |
|---|---|---|---|
| S01 | Value-only edit/re-solve; binding | Rebind values and affected value products; retain lawful immutable structure and artifacts | `PreparedCase::rebind` and complete artifact keys support this distinction. Current canonical preparation can nevertheless reacquire selected source and fresh generic admission |
| S02 | More variables/rows with one numerical policy; instance growth | Policy interpretation once, then indexed coordinate projection | Normalization and physical tolerance projection each perform per-coordinate linear searches: PE01 |
| S03 | Larger solve/fit output with the same scientific schema; instance growth | Physical blocks chosen by bounded payload/index extent, with complete fenced coverage | Scalar width determines very small sequential IPC blocks: PE02; acknowledgments/read crossings add PE03 |
| S04 | Many durable study points over one revision; composition/binding | Preserve distinct occurrences and seeds while amortizing unchanged source and structure | Ready-candidate path reopens source and uses fresh selected compiler workspaces: PE04 |
| S05 | Selective reopened results, long trajectory or analysis; representation | Reduce by output/range metadata before payload; preserve protection and exact completion | Existing selective indexes are strengths. Two protected calls per block remain: PE03 |
| S06 | Native provider/build input changes versus unchanged invocation; mechanism | Re-establish affected installation/ABI validity at a justified lifetime | Full cache-file hashing under exclusive lock repeats on ordinary preparation: PE05 |
| S07 | Unrelated test/backend edit before Python/worker build; mechanism | Preserve complete outer source observation without recompiling consumers whose executable closure is unchanged | Shared compiled all-tree attestation changes: PE06 |
| S08 | Add a unit/property model; domain instance or genuine concept | Existing semantic owners supply admission, balances and diagnostics; genuinely new physics changes its owner | No source-established reason to replace the governing physical/scientific model |
| S09 | Substitute evaluator/sparse/native capability; mechanism | Integration owner absorbs compatible changes; capability differences remain visible | Library-owned math, sparse factors and native solve boundaries are suitable. Conditional opportunities require exact contracts |
| S10 | Cancel, lose acknowledgment or restart; lifecycle | Drain actual work; preserve truthful retained effects; recover only necessary units | Existing fences, exact operation receipts and protection remain constraints on batching/reuse remedies |

Recycle, dynamics and fitting are assessed as consumers of the same preparation, numerical policy, attempt and result contracts. Their full scientific campaigns were not re-executed. No source finding here asserts that every such workflow has the same latency driver.

## 5. Execution mechanisms

| Stage | Structure/value inputs; reuse | Effects/lifetime | Evidence and material limits |
|---|---|---|---|
| Source selection | Exact revision, selected roots and complete dependency frontier | Protected reads, grouped membership/name/object acquisition | **Implemented:** current frontier grouping and parsed-once reference inventory; no ordinary full-source export claim |
| Generic compiler admission | Selected rows, physical scope and documents | Fresh canonical compiler workspace per preparation | **Implemented:** fresh workspace causes selected generic checking to repeat; retained body artifacts can still hit |
| Structural and value preparation | Structural view, bindings, profile and provider/environment identity | Immutable products versus value-dependent facts | **Implemented:** explicit rebind and selected artifact keys; no global outer-build key in current artifact requests |
| Evaluator compilation | Demanded outputs, active derivative coordinates and optimizer/evaluation limits | Effectful compilation and bounded shared templates | **Implemented:** multi-output optimization and vectorization; library already performs post-vectorization stack optimization |
| Native solve | Prepared class, values, contextual controls and permitted starts | Owned native worker/session; cancellation holds charges through drain | **Interface-checked:** existing class-specific native integration and retained-session mechanisms; consumption varies by workflow |
| Original assessment | Actual candidate, original oracle, physical budgets and requested engineering evidence | Fresh numerical-state checks before scientific permission | **Implemented:** original-space quality and engineering accuracy owners; these checks are not removed |
| Publication | Exact IPC, indexes, fence and ordinal sequence | Bounded append, close/reconciliation, short final visibility boundary | **Implemented:** sequential small blocks and full-payload acknowledgment; PE02/PE03 |
| Reopened results | Exact run/attempt, descriptor, metadata and requested range/output | Protection owns database lifetime; copied arrays own memory lifetime | **Implemented:** selective discovery, followed by metadata and payload calls; PE03 |
| Native setup | Selected pins, provider ABI, installed files and build inputs | Exclusive cache validation/build publication | **Interface-checked:** full receipt-file verification and eager family setup; PE05 |
| Outer build identity | Complete dirty outer sources, locks and configuration | Compiled attestation consumed by Python and optional xtask roots | **Implemented:** broad attestation invalidation; PE06 |

### Numerical stage contracts

| Numerical stage | Formulation/guards | Derivatives and scaling | Class/capability | Outcome and checks |
|---|---|---|---|---|
| Mathematical preparation/evaluation | Original obligations and lazy branch regions survive rewriting | Symbolica/Numerica exact admitted derivative route; active coordinate support; physical type admission | Evaluator capability, not a solver | Typed domain/provider failure; finite evaluation and construction bounds |
| Square/NLP execution | Original problem and any admitted derived representation remain distinct | Declared derivative order; contextual normalization and row-budget transport | Class-aware native route and selection | Typed native termination; fresh original residual/bound/quality checks |
| Implicit response | Selected root/sheet, guard and regularity scope | Actual current Jacobian, retained symbolic sparse pattern, original derivative checks | faer factors and library solve actions | Singular/domain/unsupported response remains unavailable or refused |
| Dynamics/estimation consumers | Declared integration, event or fitting meaning | Their admitted derivatives and contextual controls | Existing class-specific integration/optimization owners | Partial trajectories and uncertainty remain explicit; full conformance not rerun |
| Engineering accuracy | Same actual point/source and independently consumed output demand | Frozen physical scales; exact/estimated/certified evidence distinguished | Original response/arithmetic capabilities | Unavailable evidence is retained honestly, not replaced by tighter unrelated stopping |

The reviewed opportunities remove lookup, allocation, admission or transport work. They do not remove numerical refactorization when values change, original-state checks, or independently required scientific evidence.

## 6. Foundations and gates

### Architectural fitness

| Foundation | Verdict | Evidence / required action |
|---|---|---|
| AP-01 Separation of concerns | Satisfied in inspected scope | Scientific meaning, native iteration, structural persistence and result representation have identifiable owners. Preserve them when fusing physical operations |
| AP-02 Stable contracts | Satisfied in inspected scope | Explicit capability, failure, projection and lifecycle contracts. Remedies must retain their consumed semantics |
| AP-03 Composition | Satisfied in inspected scope | Studies, cases and numerical workflows compose shared owners. Repeated physical preparation is PE04, rather than evidence that occurrence policy needs replacement |
| AP-04 Adequate domain model and semantic authority | Satisfied at interface/source level in inspected scope | Physical/contextual/branch/outcome distinctions govern behavior. This does not qualify all scientific families |
| AP-05 Explicit structure and constraints | Satisfied in inspected scope | Checked revisions, roles, limits, capabilities, fences and protection are explicit. Increasing physical units requires replacement admission |
| AP-06 Local reasoning/testability | Satisfied for examined owner boundaries | Pure compiler/math/policy contracts exist. Broad runtime fixtures deserve narrower use where the tested responsibility permits it, but their whole-test timings do not establish a production defect |
| AP-07 Execution fits workload | Violated | PE01–PE04 impose avoidable repeated lookup, framing, transfer and admission under S01–S05. PE05/PE06 require additional lifecycle/provenance decisions |

### Behavioral and scientific gates

“Pass” below applies to the examined design/operation contracts at the stated source evidence level; it is not an assembled test pass.

| Gate | Judgment | Evidence / limit |
|---|---|---|
| G1 Authority | Pass in inspected scope | Authored physics and resolved numerical policy govern derived products; no new competing writable scientific authority established |
| G2 Semantic fidelity | Pass in inspected scope | Physical distinctions, exact payload coordinates and typed outcomes are explicit; proposed changes must preserve them |
| G3 Validity | Unresolved for assembled scientific scope | Admission and original checks have implemented routes, but this review does not establish validity across the unfinished scientific campaign |
| G4 Hidden behavior | Pass in inspected scope | Native effects and canonical publication remain outside pure tracked mathematics |
| G5 Consistency/recovery | Pass at examined contract level | Fencing, exact operation settlement, protected reads and closed complete admission exist; batching remedies need their own lifecycle controls |
| G6 Transformation/reuse | Pass at examined contract level | Complete selected dependencies, explicit structural/value separation and request-owned artifacts; PE04 is repeated work, not established unsound reuse |
| G7 Truthful capability claims | Pass for claims made here | No speedup, complete qualification or scientific failure inferred from the interrupted run; unsupported numerical evidence remains explicit |
| G8 Library leverage | Pass in examined scope | Libraries own arithmetic, derivatives, sparse factors, structural algorithms and native iteration. Conditional opportunities below do not establish generic reimplementation |
| G9 Architectural fitness | Fail | AP-07 fails under the representative production growth/reuse scenarios |
| PS-G1 Physical validity/conservation | Unresolved for assembled scope | Governing physical and closure contracts examined; full model/provider conformance not newly established |
| PS-G2 Well-posedness/structure | Pass at examined design-contract level | Explicit roles, incidence, matching and class-aware admission; numerical rank remains separate |
| PS-G3 Numerical integrity | Unresolved for assembled scope | Guards, derivatives, contextual controls and original assessment have source routes; interrupted qualification does not settle all numerical behavior |

## 7. Findings

### <a id="pe01"></a>PE01 — Numerical coordinate projection performs quadratic identity lookup

**Principles / gates / scenarios:** AP-07, DP-10; G9; S02.  
**Priority:** High.

**Implemented diagnosis:** `Normalization::from_policy` in `crates/pse-math/src/normalization.rs` maps every variable and row through `policy.targets.iter().find(...)`. `Tolerances::from_policy` in `crates/pse-backend-native/src/quality.rs` independently repeats that pattern for physical budgets. `ModelingCaseResolution::assess_route` in runtime `workflow/modeling/cases.rs` consumes both projections.

With similarly sized coordinate and target inventories, projecting the complete coordinate set repeats a growing full-vector search. This is identity lookup work, not physical assessment, numerical iteration or a necessary global computation.

**Proposed correction:** Give the resolved-policy owner one admitted `(target kind, semantic ID)` access structure, or produce a coordinate-ordered projection once for the consuming case layout. Normalization and quality then consume that projection. Use ordinary library containers; no new policy service or planner is needed.

Preserve contextual characteristic scales, frozen budgets, integer normalization, provenance, missing-target refusal, duplicate/conflict handling and caller coordinate order. Do not index by ID alone when kind participates in meaning. A legitimate projection containing only some targets must not lose other targets needed by closure or engineering accuracy.

**Alternatives:** A temporary index per projection removes the quadratic scan with minimal contract change; a shared immutable index or shared coordinate layout amortizes it across consumers. Binary search is viable only with a maintained canonical ordering and correct kind/ID key.

**Settling evidence:** Permuted coordinate order, missing targets, conflicting duplicate entries and same-ID/different-kind controls should retain behavior. Source should show one index construction or ordered join followed by bounded lookups. A representative end-to-end measurement can establish latency benefit; none is claimed here.

### <a id="pe02"></a>PE02 — Scalar-index count forces tiny independently framed result blocks

**Principles / gates / scenarios:** AP-07, DP-10, DP-19–DP-21; G9; S03/S05.  
**Priority:** High.

**Implemented diagnosis:** Runtime `workflow/result_projection.rs::store_result_table` computes `chunk = 64 / scalar_width`. Solve variables have width ten and constraints width eight, so they are sliced into six and eight rows respectively. `result_blocks::visit_result_blocks_async` awaits each visitor; each visitor calls canonical append. Each block is a self-contained IPC stream with schema and end marker, independently of the 512 KiB payload bound.

`append_execution_batch` and the generated append function bulk-insert cell arrays. There is no per-cell RPC. The amplification is that a small scalar-index admission unit also dictates IPC framing, transaction and acknowledgment granularity.

**Proposed correction:** Choose scientific IPC units using bounded payload, decoded extent and index metadata, rather than fixing the payload unit to 64 scalar cells. Admit a bounded larger index array, or decouple index staging from payload framing under a single exact block/attempt admission. Prefer the simpler bounded append when it preserves the full contract.

Preserve exact original rows, every declared indexed field, cell-to-block correspondence, ordinal continuity, atomic payload/index visibility, finite reservations, fenced cancellation and exact conflicting-replay detection. A wide indivisible schema must still refuse when it cannot fit; larger batches do not solve that case.

**Alternatives:** Larger bounded append units; bounded multi-block append preserving individual IPC blocks; separately staged indexes sealed with the payload. The last option adds recovery machinery and is justified only if simpler bulk admission cannot meet the bounds.

**Settling evidence:** Boundary-sized and empty blocks, mixed missing values, exact signed-zero bits, ordinal gaps, late writes, lost acknowledgments, conflicting retries and cancellation must retain their meanings. Inspect actual blocks/calls for a representative solve result; measure the complete publication route before claiming a speedup.

### <a id="pe03"></a>PE03 — Successful result crossings carry more work than their guarantees require

**Principles / gates / scenarios:** AP-07, DP-10, DP-19, DP-21; G9; S03/S05/S10.  
**Priority:** High for the combined publication/read route.

**Implemented diagnosis:** The generated append function in `pse-codegen/src/codegen/surreal_execution.rs` returns the complete saved `canonical_result_batches` record. Rust `append_execution_batch` decodes and compares it with the original batch, including IPC bytes. Normal success therefore echoes the complete scientific payload.

Separately, `canonical_results.rs::result_block` fetches and checks exact recorded block metadata through one protected transaction, then calls `result_payload`, which performs another protected transaction for the batch. The two checks can remain necessary while their two RPC boundaries are unnecessary.

These are related crossing costs with **distinct closure obligations**: append acknowledgment must establish exact operation settlement; read composition must establish metadata/payload correspondence under protection.

**Proposed correction:** Return a sufficient immutable acknowledgment for normal append success, while retaining server-side byte comparison and exact operation/request identity. Reserve full readback for uncertainty where necessary. Compose metadata and payload acquisition in one protected bounded operation, or acquire a bounded set of exact blocks while checking each correspondence.

Do not equate a matching digest with scientific correctness, discard conflicting replay checks, treat transport success as statement completion, or cache expired read authority. A forged/mismatched metadata request must still refuse.

**Alternatives:** Keep single-block reads but fuse their protected statements; group exact blocks under a combined byte reservation; retain current full payload readback only on uncertain acknowledgment. Independent acknowledgments and reads can be corrected separately.

**Settling evidence:** Exact repeat versus changed payload/metadata, lost response after commit, missing operation receipt, expired protection, reclamation, wrong attempt/ordinal and incomplete SDK statement results. Source should show removal of the normal payload echo and redundant protected crossing, not relocation into another layer.

### <a id="pe04"></a>PE04 — Durable ready occurrences reacquire unchanged selected source and generic admission

**Principles / gates / scenarios:** AP-07, DP-09/DP-10, PS-11; G9; S01/S04/S10.  
**Priority:** High for studies and repeated case preparation.

**Implemented diagnosis:** `workflow/study.rs::prepare_study_candidate` reads the point/study, reopens exact physical/modeling sources and prepares the bound operation for each ready occurrence. The declared-case path reaches selected source preparation. `ModelingPackage::prepare` acquires `selected_source`, constructs `canonical_workspace` and calls `admit_source_in`.

`MathService::canonical_workspace` in `math.rs` always constructs a fresh `CompilerWorkspace` with a request-bound canonical retention attachment. `admit_source_in` calls `modeling_revision`, whose fresh workspace enters `publish_modeling_with` without a prior selected modeling state and performs generic checking.

This does **not** establish that every body, evaluator or native compilation repeats. Physical admission caching, shared body/artifact retention and durable descriptions are real strengths. It establishes that retained mathematics can be preceded by repeated selected hydration and generic admission. In-process study/session reuse is not evidence that the durable ready-occurrence path amortizes that work.

**Proposed correction:** Retain bounded immutable checked selected products or a worker-local compatible preparation group at the source/structure lifetime. Bind occurrence values and provenance separately. The owning source/preparation mechanism must obtain fresh storage protection and recheck complete dependency/interpretation/producer eligibility before reuse becomes consumable.

Do not retain request-specific `SelectedRead` authority indefinitely or share mutable attempt state. Equal bindings must not merge occurrences, claims or attempts. Changed membership, missing-name resolution, physical context, providers, policy or relevant source must invalidate the affected product. A restart can reconstruct the product from canonical descriptions; a process-local hit is only an accelerator.

**Alternatives:** Bounded immutable selected admission reuse; prepare a bounded compatible ready group; use persisted compiler-issued descriptions to skip known admission reconstruction where their complete contracts permit it. Simply adding another hash cache without ownership, protection and invalidation would not close the finding.

**Settling evidence:** Repeated ready points over unchanged source; value-only versus structural changes; A/B/A selections; additions/deletions/shadowing; changed physical/store/session owner; clear during in-flight preparation; cancellation and expired/reclaimed sources. Compare fresh and reused products, including attribution and numerical policy. Demonstrate which admission work disappears while required protection remains.

### <a id="pe05"></a>PE05 — Native installation assurance repeats complete verification under exclusive preparation

**Principles / gates / scenarios:** AP-07, DP-10, DP-20; G9; S06.  
**Priority:** Medium; remedy depends on the installation trust/lifetime decision.

**Implemented diagnosis:** `scripts/native_cache.py::prepare` holds an exclusive per-identity lock while `valid` hashes every recorded installed file. Repeated native environment preparation re-establishes the complete header/library receipt. `native_pipeline_cache.prepare` also discovers the HiGHS Cargo archive before deciding the Uno cache hit and hashes provider archive/header/BLAS inputs. The shared native environment sources solver and math preparation, including KLU, isolation and pipeline families.

Complete linked qualification legitimately needs its consumed family. The source does not establish that every enabled provider can be omitted from an arbitrary linked target. The concrete concern is repeated preparation/verification at command lifetime and broad setup where a narrower consumed closure is available.

**Proposed correction:** Define a controlled verified-installation lifetime and capability-specific setup closure. Full verification occurs when admitting/changing an installation or crossing the relevant trust boundary; compatible consumers reuse that established lifetime. Construction/publication exclusion remains separate from read-only use. Avoid the preliminary Cargo/provider discovery when an already admitted exact provider installation can supply the same actual identity.

Immutability, corruption detection and provider-change invalidation must be settled first. Existence-only checks, unchecked explicit prefixes or a permanent “verified” flag do not preserve the guarantee.

**Alternatives:** Retain current verification where uncontrolled mutation is part of the supported exposure; amortize verification within a verified session; introduce controlled immutable installations with explicit revalidation. Narrow target setup independently where actual feature/link closure permits it.

**Settling evidence:** Changed/corrupt/missing files, simultaneous first preparation, interrupted build, provider/archive/header changes, explicit prefix handling and actual ABI association. Comparable cold/warm setup/build observations should retain checkout, target, toolchain and provider configuration. No share of the observed build time is attributed to this mechanism.

### <a id="pe06"></a>PE06 — Complete outer source attestation broadens compiled consumer invalidation

**Principles / gates / scenarios:** AP-07, DP-09/DP-10, DP-21; G9; S07.  
**Priority:** Medium; requires a provenance contract decision.

**Implemented diagnosis:** `pse-buildinfo/build.rs` and `identity.rs` watch/read all files beneath `crates`, `vendor` and `xtask`, then embed the outer source identity. An unrelated test/backend edit changes this compiled attestation even when a receiving executable’s result-affecting unit closure is unchanged.

Current direct consumers are `pse-py` and the optional xtask worker/deployment composition. Compiler/runtime manifests no longer directly consume it. This is remaining build amplification; it is **not** evidence that current scientific artifact keys use the whole-tree hash.

**Proposed correction:** Separate complete dirty-tree observation from the artifact-specific source/build identity whose association must be established for the receiving executable. Place complete outer observation at the actual deployment/run boundary, associated with the observed executable and reviewed producer receipt. Preserve actual source/build association and the full dirty outer inventory; reduce unrelated compiled dependency fan-out.

This recommendation changes a current linked-attestation interpretation and needs RC01 confirmation. A runtime-manufactured matching JSON pair is not sufficient artifact evidence.

**Alternatives:** Retain the linked all-tree identity and accept its build cost; reduce repeated inventory reading without changing its genuine invalidation; or adopt the separated artifact/outer-observation contract. Only the last removes the unnecessary compiled fan-out.

**Settling evidence:** Actual Python/worker artifact association, unrelated outer edit, relevant producer edit, changed generated/native/configuration input and deliberately mismatched receipt. Comparable build observations must distinguish inventory work, recompilation and relinking. No numerical build speedup is claimed.

## 8. Library fit and additional opportunities

The [math evidence](../evidence/production-execution-efficiency-2026-10-07/math/README.md) and [SurrealDB evidence](../evidence/production-execution-efficiency-2026-10-07/surreal/README.md) identify exact versions, source backing, documentation transfer and limits.

| Capability | Fit / ownership | Recommendation and costs |
|---|---|---|
| Symbolica/Numerica evaluation | Multi-output optimization, active coordinate selection, guarded stages and attempt workers already exist | Preserve this integration. Symbolica 3.0.1 vectorization already invokes stack optimization; adding another call is not an established improvement |
| Structural Taylor zeros | Current `Dualizer::new` receives no zero-component list; library can suppress proven-zero component work | **Proposed, conditional:** derive zeros from complete per-input structural support. Preserve tuple order, ancestor-closed Taylor shape, providers, branch/control dependencies and admission. Numerical zero at one point is insufficient |
| faer sparse factors | Symbolic analysis is retained; current high-level numeric refresh constructs numeric factor/scratch storage | **Proposed, conditional:** retain numeric buffers/scratch inside the compatible attempt owner. Still factor the current Jacobian and preserve pivot/rank/nonfinite recovery and backward-error checks |
| Salsa | Current tracked inputs, durability, equality/backdating, LRU and bounded generation rebuild are used | No second incremental engine or automatic persistence rewrite. Finer checked-catalog inputs are conditional on actual edit workload benefit and complete package/visibility checks |
| Native Ipopt/POUNCE and other class routes | Established libraries own iteration, factors and native model management | Preserve original-space assessment. Existing session implementations do not establish their consumption by every durable journey; examine that lifecycle before extending retention |
| SurrealDB 3.3.0 | Appropriate structural transactions, exact identities, bulk writes and selected metadata | Correct PE02/PE03 in the existing integration. Do not add another retry layer or move scientific mathematics into database functions |
| Selected source access | Useful indexes and grouped pages exist; supplier predicates have correlated-work exposure | **Conditional:** inspect exact bound active/historical/supplier variants before selecting compound indexes or target-driven acquisition |
| Arrow/DataFusion | IPC preserves admitted scientific rows; selective canonical metadata precedes Arrow analytics | Preserve filter/projection/completion semantics. A custom provider is conditional on a real analytical consumer and protected pushdown contract, not merely library availability |

SurrealDB 3.3.0 tries the streaming planner first and folds eligible parameter constants before index analysis. Optional-OR syntax alone therefore does not establish a table scan. Bounded returned pages also do not establish bounded examined work. Actual access choices remain a material uncertainty for source-growth remedies, settled by release-compatible plan inspection over selective, empty, historical and skewed cases.

No library replacement is justified solely by the available whole-test timings. Graph-native storage does not replace numerical kernels, and a library name does not establish efficient complete execution.

## 9. Alternatives and compatibility of remedies

| Alternative | Benefit and burden | Decision |
|---|---|---|
| Current guarded canonical baseline | Strong semantic/lifecycle separation; retains repeated projection, framing, transfer and admission | Preserve guarantees; revise the identified physical units |
| Indexed policy plus bounded bulk result operations and reusable immutable selected admission | Removes concrete work without changing the scientific authority or numerical algorithm | Preferred proposed direction for PE01–PE04 |
| Library-owned Taylor/sparse-storage improvements | Can reduce derivative arithmetic/allocation in applicable paths; requires exact support/lifetime integration | Conditional follow-up after identifying the relevant consumer |
| Replace SurrealDB or add an alternate production store | Introduces codecs, migration, recovery and qualification; does not itself fix lookup or tiny result units | Not supported by current evidence |
| Simplest local implementation corrections | Temporary ID index, larger bounded append, fused protected read and scoped admitted product retention | Prefer where they discharge the contract without another framework |

PE01 and PE02/PE03 are independent. Larger result blocks reduce the number of crossings, but acknowledgment/read corrections remain useful independently and require distinct controls. PE04 amortizes source/structural preparation; it must not become another independently writable dependency authority.

PE05’s prerequisite is an installation validity/lifetime decision. PE06’s prerequisite is an artifact-versus-outer-attestation decision. Their priority is lower than the directly demonstrated numerical/result/study amplification, although those contracts must be settled before implementing their remedies.

No general cache, scheduler, planner, telemetry framework or cost model follows from this review. Short bounded observations are appropriate only when they can choose between materially different remedies.

## 10. Verification and evidence limits

[Preserved execution evidence](../evidence/production-execution-efficiency-2026-10-07/execution-baseline.md) owns the historical observations and attribution limits.

**Measured, preserved historical observation:** `build/assessment/plan28-qualified-resumed-20261007/` contains the interrupted native gate. The command was `just native-test --profile local --config-file <run>/native-test-nextest.toml`, in native force-validation mode. Failure baseline is zero.

Of 2,821 selected tests, 2,425 passed, one was aborted by signal 15 and 395 did not run. The maintainer requested the interruption. The aborted progress-batching control is not an established scientific failure. The enclosing report has 39 positive gates, one interrupted gate and one not-run gate; 37 positive gates were explicitly transferred from their prior qualified source context. The Python series did not run.

The native gate’s 1,793.689 seconds include the logged four-minute-eleven-second test build. The 284.005-second flash observation and roughly 35-second small-result observations are whole-test times with setup/body/teardown unsplit. They do not measure isolated solve or indexed-query latency. Fixture database creation, full schema installation, explicit additional preparation and teardown are material attribution limits. Nextest serialization is test scheduling evidence, not a production-code speed remedy.

| Claim | Evidence | Result / settling obligation |
|---|---|---|
| PE01 repeated target searches | Implemented, inspected callers | Structural diagnosis established; indexed correction and numerical equivalence controls Proposed |
| PE02 small sequential result blocks | Implemented, inspected projection/framing/append | Structural diagnosis established; larger physical unit and complete publication measurement Proposed |
| PE03 echo/two-read crossing | Implemented, generator and Rust consumers inspected | Structural diagnosis established; acknowledgment/read correctness controls Proposed |
| PE04 fresh selected admission | Implemented, durable candidate → selected preparation → fresh workspace inspected | Repeated hydration/generic admission established; no claim that all arithmetic compilation repeats |
| PE05 complete native verification | Interface-checked, scripts inspected | Repeated mechanism established; safe installation lifetime remains a design decision |
| PE06 outer attestation fan-out | Implemented, build source and direct manifests inspected | Current narrowed consumer scope established; separated association contract Proposed |
| Source-query selectivity | Interface-checked library/source evidence | Actual selected access path and examined work unresolved; no scan defect asserted |
| Taylor-zero/factor-storage benefits | Interface-checked version-backed alternatives | Consumer applicability and quantitative benefit unresolved |
| Scientific conformance | Source contracts plus preserved partial/historical evidence | Full affected unit/property/dynamics/fit conformance not newly established |

No production test or benchmark was run for this review. Model-specific reference comparisons retain their original selected physical parameters, conventions and conditions. The review grants no new numerical IDAES equivalence or whole-simulator scientific qualification.

## 11. Rule impacts and disposition

Recommendations are judged before comparison with current rules. No rule change takes effect through publication of this review.

### <a id="rc01"></a>RC01 — Separate linked executable identity from complete outer source observation

**Current rule:** ADR-0164’s deployment qualification binds reviewed actual tool capture to the linked executable outer attestation; `pse-buildinfo` embeds complete outer sources. Related owners are blueprint §5/§20 and Plan 28 B3/E producer association.

**Proposed change:** Preserve an artifact-specific linked identity and actual observed artifact association, while recording complete dirty outer source observation at deployment/run admission. Specify their relationship explicitly.

**Depends on:** PE06.

**If retained:** PE06 becomes an accepted build tradeoff only with its concrete provenance benefit and cost stated; improvements are limited to inventory implementation and scope-compatible build mechanics. Unrelated genuine all-tree changes still invalidate the compiled attestation.

**Route:** Operator confirmation in plan creation, then the appropriate identity/deployment ADR/design amendment and actual producer-association controls. This review does not weaken attestation or apply the change.

### Other rule effects

PE02 changes the current 64-cell append admission and replay-comparison implementation contract. Its replacement bound must be explicit and versioned where durable interpretation requires it. The 512 KiB self-contained result-block contract can remain. No rule requires IPC framing to follow six/eight-row units, so that guarantee need not be discarded.

PE01, PE03 and PE04 can preserve current semantic authority, exactness, protection, complete dependencies and attempt contracts. PE05 needs a concrete installation trust/lifetime decision before verification frequency changes; it grants no permission to skip current checks in the meantime.

No SHOULD exception is requested to excuse the AP-07 MUST defects. No changes are recommended to scientific tolerances, test concurrency/timeouts, the selected server profile, clean-room reference practice, library eligibility or exact dependency policy.

All PE findings link to Plan 28’s coordinator as their single proposed disposition owner. Adoption, scheduling and current status belong there, with evidence or a revisit trigger. This review does not schedule implementation.

## 12. Decision and next consequential choices

**Architectural fitness: Revise**, because the production route contains source-established AP-07 amplification under ordinary coordinate/result/study growth. **Behavioral and scientific adequacy:** examined contracts have substantial implemented protection, but assembled scientific acceptance remains unresolved and the interrupted run supplies no inferred scientific failure. **Overall: Revise.**

| Priority | Change | Findings / scenarios | Acceptance evidence | Owner |
|---|---|---|---|---|
| High | Indexed numerical-policy projection | PE01 / S02 | Preserved coordinate/budget semantics and removed repeated full-vector search | Plan 28 coordinator |
| High | Result physical units and sufficient crossings | PE02/PE03 / S03/S05/S10 | Exact rows/indexes/replay, complete coverage, cancellation and protection; complete-route observations | Plan 28 coordinator |
| High | Amortized immutable selected admission for repeated work | PE04 / S01/S04/S10 | Fresh-versus-reused equivalence, complete invalidation, attribution and renewed protection | Plan 28 coordinator |
| Medium | Verified native installation lifetime and scoped setup | PE05 / S06 | Corruption/change/ABI and concurrent preparation controls; comparable setup observations | Plan 28 coordinator |
| Medium | Artifact/outer-attestation decision | PE06/RC01 / S07 | Actual installed artifact association and relevant/unrelated edit controls | Plan 28 coordinator |
| Conditional | Query access, Taylor zeros and numeric scratch | S05/S09 | Evidence selecting the relevant consumer/remedy, followed by its semantic controls | Plan 28 coordinator |

Priority follows consequence; prerequisite order follows the guarantee required by the remedy. PE02 needs a replacement bounded admission before larger blocks. PE03 needs a sufficient exact acknowledgment before removing full readback. PE04 needs a reusable immutable witness with fresh protection, rather than retained stale request authority. PE05 and PE06 require their explicit lifetime/provenance decisions first.

The next design choices should select those physical boundaries and preservation contracts. Proposed speedups remain unmeasured. Production implementation and subsequent qualification remain separate authorized work, with E4/E5 still owned by 28e.
