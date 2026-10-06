# Plan 28 completion audit

**Date:** 2026-10-06  
**Tier / purpose:** Design-tier target assessment, with a separate implementation/evidence completion audit.  
**Boundary:** Plan 28 R0 and A–E, accepted RC01–RC10, US01–US05, EF01–EF08, and the applicable scientific qualification transferred from Plans 25/27.  
**Standard:** Core 3.4, Efficient Architecture Heuristics 1.0, ProcessSimulator 1.5 and the [repository binding](../design_principles/binding/pse-arrow.md).  
**Baseline:** `06302af7748923100f17a7c8fda9c57b6a536690`, initially clean. Concurrent documentation edits appeared first; subsequently the working tree changed across production source, tests, generated outputs, manifests and tooling. Those changes were preserved. This review assesses the stated commit, not acceptance of the newer working tree. The audit changed no production source.  
**Review method:** Independent A/B implementation review and bounded assembled architectural judgment; independent C/D code/evidence mapping; coordinator inspection, execution and synthesis. The independent reviewer did not implement Plan 28. Thread limits prevented a further separate design-reviewer session; the reviewer examined A/B deeply and C/D seams directly, and this document does not claim whole-simulator architectural acceptance.  
**Disposition owners:** [Plan 28](../../plans/28-surrealdb-unified-substrate.md#finding-dispositions); [28e](../../plans/28e-rebuild-retirement-and-qualification.md) owns assembled qualification. Existing contextual-accuracy findings remain at [25k](../../plans/25k-integrated-qualification-and-closure.md#contextual-accuracy-review-dispositions). This report is a dated assessment, not another status ledger.

## Assessment

**Plan 28 is not complete. Architectural decision: Revise. Assembled behavioral/scientific acceptance: Not Accept.**

The pivot has substantial implementation: native canonical revisions and guarded selection, portable mathematical descriptions, automatic scientific retention, scoped studies, connected Arrow results and graph analyses. The old PostgreSQL/Delta/query-publication production paths have been removed from the inspected tracked owners. Runtime explicitly composes scientific, resource and persistence responsibilities rather than placing storage interpretation inside mathematical kernels.

Three inspected mechanisms prevent architectural acceptance: generic retention can reassign lifecycle-owned source roots; selected compilation repeatedly performs small serialized namespace and payload operations; and fresh schema creation can accept a response that the normal canonical transaction wrapper rejects as incomplete. The first is a source-preservation defect; the second is avoidable preparation amplification; the third is a missing initialization-success invariant. Their corrective directions fit together and require no change to the approved scientific target.

Fresh partial verification records failures against the zero baseline; source drift subsequently invalidated assembled qualification. Deployment-qualified Python reuse, comparable measurements and closure remain unfinished. The commit title “plan 28 done” does not settle any of these obligations. Pending plan checkpoints are useful leads; the verdict follows source and execution evidence.

### What is preserved

The inspected vocabulary distinguishes problem/revision, versioned membership, preparation/product, relevant producer, complete build attestation, run/attempt, closed manifest, read protection and retention owner. Selected physical and declaration inputs still enter shared scientific admission. Reconstruction uses concrete owning-kernel receipts, not printed CAS expressions or serialized native factors. Compact formals and explicit gathers preserve native evaluator/derivative ownership. Results distinguish failed/partial/cancelled outcomes from scientific success. Copied decoded Arrow arrays retain accounted memory without needing an indefinite database pin.

These are **Implemented** source paths. Named local controls and historical packet receipts retain their original scope; none alone demonstrates assembled qualification.

## Target, ownership and revealing changes

The target is ordinary process modeling with selected sufficient scientific inputs, retained outcomes that can be reopened exactly after restart, and studies whose workers preserve occurrence and predecessor meaning. The important drivers are scientific authority, guarded interpretation, recoverable lifecycle effects, bounded memory and preparation that follows necessary input. The inspected deployment is one local supervised server with native clients, not a distributed availability architecture.

Canonical operations own revisions, exact encoded source and graph relationships, protections, roots and transaction completion. Modeling owns scientific dependency discovery and sufficient admission; mathematical kernels own reconstruction, evaluation and derivatives. Runtime composes those owners with attempt-scoped numerical state, durable ingestion and result admission. Python owns boundary conversion, while Arrow is an independently owned result representation. A local retention change needs lifecycle-root consumers, not a solver rebuild in conceptual terms; a scientific provider change needs its interpretation and receipt contracts, not a second database policy.

The [adopted representative journeys](design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#representative-journeys) remain the scenario authority. These additional review scenarios explain the decisive mechanisms and credible variation axes:

| Scenario | Expected boundary and observed consequence |
|---|---|
| S01: a retained R1 run remains available while authoring R2 and forgetting deliberate history | Revision history and run retention can vary independently. F01 instead lets a generic retention operation redirect a lifecycle root, violating that separation. |
| S02: extend a selected model with many legitimate scalar declarations, then change one nearer name or provider | This is a domain extension using existing primitives. Selected scientific closure remains necessary, but singleton requests and accumulated rescans grow with incidental operation boundaries (F02). Exact absence/shadowing and provider guards must survive grouping. |
| S03: replace the canonical driver's response-delivery mechanism or interrupt initial installation | This is a mechanism substitution, with completion meaning preserved. F03 exposes an initialization path that consumes weaker evidence than normal queries. A new driver must not establish writes from an unconfirmed response. |
| S04: two workers claim distinct ready occurrences while one predecessor changes and a committed acknowledgment is lost | Study policy owns readiness and exact premises; the database owns guarded claims, runtime owns attempt recovery. Focused claim controls establish part of this contract, but lost-ack child-process execution remains unsettled. |
| S05: read several Arrow pages while result/source retirement competes; retain an escaped decoded array | Canonical pins cover reads and source/result admission; copied Arrow allocations own memory afterward. Separately positive controls do not settle the combined retirement/read/analysis lifecycle. |
| S06: build a new installed Python extension and reopen a portable product | Relevant producer compatibility may remain stable while full attestation changes. Actual imported-artifact association is required; a finite fixture or another target's receipt cannot establish it. |

### Library fit, alternatives and tradeoffs

SurrealDB native transactions and structural functions can carry guarded revision, claim and admission operations directly. The integration owner must preserve statement completion and explicit uncertain outcomes; snapshot isolation is not proof of predicate serializability. These needs justify domain-specific native operations, not an interchangeable generic store API that erases their premises. This audit is source inspection of consumed contracts, not independent qualification of every SDK or RocksDB behavior.

Arrow IPC and DataFusion memory reservations provide existing representation/allocation mechanisms. Persisted blocks require exact metadata, receipt checks and bounded decoding; decoded arrays need their own reservation after database protection ends. Symbolica/Numerica and the class-specific solvers continue to own numerical machinery. Portable descriptions need strict reconstruction contracts rather than copied native factors or a bespoke mathematical evaluator. The audit does not recommend a numerical-library replacement.

For F01, an immutable lifecycle association in the existing native operation is the simplest viable correction. Restricting generic mutation to deliberate history is an alternative API boundary; adding a separate retention service would duplicate authority. Legitimate history movement must remain possible, and lifecycle minting must stay atomic with the owning operation.

For F02, bounded grouped frontier/extent operations use bulk capabilities already exposed by the library and operations owner. That is preferable to parallelizing the same redundant probes, hydrating the complete unrelated source bundle, or building a new preparation planner without a concrete need. Larger batches carry more temporary state and retry scope, so exact premises and memory bounds constrain grouping. The expected benefit is fewer repeated crossings; its magnitude is unmeasured.

For F03, reuse the existing complete-response contract and verify installation. Treating a transport return as success is simpler mechanically but loses the necessary completion guarantee. Blind retry can mask uncertainty and is not selected. None of these remedies moves scientific interpretation into persistence or creates a second authority.

## Findings and decisive seams

### <a id="f01"></a>F01 — Generic retention can reassign a lifecycle source root

**Priority:** High. **Owners:** A3 retention with C1/C4 lifecycle admission. **Basis:** AP-04 authoritative realization, AP-05, G3/G5/G9.

[Generic retention](../../../crates/pse-operations/src/canonical_retention.rs) accepts Run, Analysis and ActiveAttempt owners; only Product is refused. Its RETAIN transaction checks an existing root’s problem and owner, then overwrites revision/sequence without comparing the old selection. [Execution admission and claim](../../../crates/pse-codegen/src/codegen/surreal_execution.rs) create and consume the same run and active-attempt root keys.

Admit a real run on R1, advance the same problem to R2, then call safe `retain_revision(R2, Run(existing_run_key))`. The actual run root now names R2 although the immutable run names R1. A subsequent claim rejects that association. If R1 history is forgotten and no other root/protection covers it, reclamation can remove inputs still required by the retained run. The equivalent reassignment is possible for a live attempt. Existing DROP_ROOT lifecycle checks do not protect this mutation.

**Implemented/source-inspected diagnosis; no new exploit test was run.** This is reachable through the typed public operation, without a privileged raw database mutation.

**Proposed correction:** make existing lifecycle root associations immutable and reserve their minting/mutation for their owning admission/retirement operations. Keep deliberate history retention distinct. Reject reassignment rather than quietly redirecting a run.

**Verification:** real R1 run/attempt → R2 edit → both reassignment attempts refuse and leave roots unchanged → forget history/reclaim → R1 remains available. Include concurrent retirement; preserve supported legitimate history changes.

### <a id="f02"></a>F02 — Selected preparation still amplifies work per declaration

**Priority:** Medium. **Owners:** B1 selected preparation and A2 protected bulk reads. **Basis:** AP-07/G9; PSE-S01/S07.

[Selected resolution](../../../crates/pse-runtime/src/workflow/modeling/canonical.rs) awaits requests individually, supplying singleton arrays to APIs that accept grouped logical/name demands. Every closure round clones/re-examines accumulated rows. [Member-demand policy](../../../crates/pse-modeling/src/selected_source.rs) requests a complete scope for every non-package declaration, including ordinary leaves. Imports and member probes are independently scheduled; names can be reprobed even after a complete parent inventory supplied them.

[Object hydration](../../../crates/pse-operations/src/canonical_staging.rs) separately reads extent, assembly metadata, blocks and closing metadata for each selected object. For ordinary nonempty single-block leaves, the inspected route yields several serialized protected RPCs per declaration in addition to repeated closure scans. The independent review derives at least seven such calls per leaf under its stated shape; this is a source operation count, **not Measured latency**.

A sufficient semantic closure is a useful improvement, but it does not justify mapping every semantic item to separate physical operations. Neither a new generic store abstraction nor indiscriminate parallelism would remove the repeated work.

**Proposed correction:** use a bounded grouped frontier, carry already-established complete namespace inventory, avoid redundant leaf probes where grammar permits, and provide grouped protected extent/payload acquisition. Retain memory reservation, exact negative/set premises, cancellation, expiry and identity checks.

**Verification:** a valid larger scalar root retains identical closure, identity, replay and numerical results while grouped acquisition removes repeated namespace work. Check nearer-name shadowing, new members, receiving-context changes and expiry. Existing E4 campaigns can measure complete-operation effects after functional qualification; no new latency threshold is imposed.

### <a id="f03"></a>F03 — Schema initialization bypasses the complete-response check

**Priority:** Medium. **Owner:** A1 canonical initialization. **Basis:** AP-05, G3/G5/G7.

[Fresh create](../../../crates/pse-operations/src/canonical.rs) sends schema DDL through `request`, calls `checked`, and returns success. `checked` accepts an error-free zero-statement indexed response. Ordinary `bounded_query` calls `complete_response`, which explicitly rejects that response as IncompleteResponse. Existing-schema creation invokes `open()`; fresh creation omits that verification. [Fixture construction](../../../crates/pse-operations/src/testing.rs) consequently returns a store on this weaker result.

**Implemented/source-inspected diagnosis:** an empty successful-looking response can establish initialization success without completion evidence. **The diagnosis is not a demonstrated explanation of the campaign’s missing-table errors.** Schema and marker are in one transaction; an explicit transaction conflict is propagated, and there is no evidence here of a partial successful commit.

**Proposed correction:** apply the complete-response contract to DDL and verify installed interpretation before admitting a newly initialized store. Resolve uncertain initialization explicitly; do not blindly replay schema installation.

**Verification:** deterministic empty-response initialization refusal, cancellation/uncertain-completion control, and normal successful create/open. The existing cancellation control exercises bounded_query, not this bypass.

### <a id="f04"></a>F04 — Assembled acceptance inputs and runner prerequisites are incomplete

**Priority:** High for completion; causes need their owning fixes. **Owners:** E1/E3, affected fixture/scientific owners and native runner.

The fresh campaign includes failures for absent imported numerical_policy, unknown quantity entity_kind, dynamics option coverage missing version, missing canonical tables, and a RocksDB transaction conflict. These have different causes. They cannot be grouped as solver convergence failure, and a failing fixture does not by itself demonstrate a production scientific defect.

Full native selection also includes the worker binary tests without PSE_WORKER_BINARY. [The worker recipe](../../../justfile) builds and supplies it; [the full-native runner](../../../scripts/native_tests.py) does not. The resulting worker-test panics are missing execution prerequisites, not evidence that the actual managed worker cannot solve. The proper child-process journey still needs its own current execution evidence.

Some numerical assertions compare against an exact optimum at tighter accuracy than the production solve requests. In the KKT sensitivity tests, primal/row sensitivities can match while objective sensitivity differs at the approximate candidate; the solve requests no output goal and comparison uses approximately 1e-6. This reproduces the inherited need to separate response correctness at the observed point from base-point forward accuracy. **Hypothesis**, not an established derivative defect.

**Proposed correction:** complete target fixture closure and runner admission in their owners, diagnose storage response/transaction failures, and make analytical checks test their actual premises. Preserve mathematical guarantees and meaningful contextual policy; do not tighten solver defaults or tune individual tests green.

The partial linked Python campaign also observed durable progress queue overflow cancelling an event simulation, version rejection occurring after malformed nested study decoding, and study cancellation identity returned as a string where the handle exposed bytes. These are observed boundary failures, not newly diagnosed common causes. The backpressure observation makes the C2 terminal-class/recovery question concrete; a durable reopen control still needs to settle its final classification. Concurrent source edits limit attribution to the final tree.

**Verification:** repaired owners’ targeted controls, then a complete current zero-failure native/Python/manifest/parity campaign, distinguishing repaired composites from clean first passes.

### <a id="f05"></a>F05 — Deployment and closure claims lack assembled evidence

**Priority:** Completion blocker. **Owners:** B3/D2 and E3–E5.

Current Python imported-artifact association and eligible deployment admission are explicitly pending in 28e. A finite producer fixture cannot qualify the installed extension. The deployment mint is a documented trusted-operator assertion; matching outer hashes alone is not proof of the correct selected target. An honest cross-target receipt control and the actual imported extension association remain relevant, without claiming a malicious-receipt defect outside that trust scope.

No current E4 measurement receipt or completed E5 closure was found in the bounded inspected paths. E4 cannot adopt smoke-only or old-stack measurements. R0 records ADR-0164 as proposed; accepted authority changes, qualification publication and document retirement remain separate decision/closure work. This audit supplies an independent assessment; it does not accept the ADR, resolve old findings or complete E5.

## Obligation coverage

This table records this audit’s evidence boundary, not live packet status.

| Obligation | Inspected result / unresolved acceptance |
|---|---|
| R0; RC01–RC10 | Authorized target/proposal and architecture revision route recorded. ADR-0164 remains proposed; formal adoption/closure not completed. |
| A1 | Supervised authenticated loopback gRPC/RocksDB profile and resources implemented; F03 initialization gap; supported kill/reopen/restore evidence must remain condition-specific. |
| A2 | Exact native codecs, immutable memberships, guarded selections/staging and operation identities inspected. Full assembled codec/recovery scope still needs a pass. |
| A3 | Protected reads/product admission/bounded reclamation implemented; F01 prevents lifecycle retention acceptance. |
| B1 | Shared selected admission exists; F02 amplification and fresh scientific input failures prevent completion acceptance. |
| B2 | Strict portable reconstruction and concrete dependencies inspected; current deployed cross-build reuse remains unqualified. |
| B3 | Relevant identity and outer attestation separated; real current runtime/worker receipts not independently located in bounded search; installed Python association pending. |
| B4 | Compact formals/gathers and actual native-owner reuse implemented in inspected consumers; no speedup inferred. |
| C1 | Run/attempt/frozen-manifest semantics and fencing implemented; lifecycle-root seam fails F01. |
| C2 | Ordinary durable registration/drained progress/table ingestion/close/reconcile/seal implemented; actual failure/partial/cancel restart qualification remains incomplete. |
| C3 | Scoped native discovery/claim/predecessor policy and distinct occurrences implemented; proper two-worker/killed-worker journeys need valid runner prerequisites. |
| C4 | Cancellation/drain, expiry reconciliation and result retirement implemented; lost-ack and assembled interruption scope unresolved. |
| D1 | Exact terminal attempt/manifest/member selection and scalar/IPC layout implemented; scientific meaning remains governed by owning kernels. |
| D2 | Renewed bounded readers, independently owned decoded arrays and staged export implemented; deployed Python admission and assembled transport/race acceptance incomplete. |
| D3 | Original incidence/dependency and retained sensitivity/provenance graph routes inspected; complete source-root/retirement interactions not qualified. |
| E1 | Target generation/fixtures migrated in source; missing input closure in fresh tests needs correction and full manifest acceptance. |
| E2 | Inspected old production crates/lowerings/dependencies removed; stale excluded empty crate directories and obsolete tooling references still fail retirement checks. |
| E3 | Fresh verification fails; no assembled zero-failure acceptance. |
| E4 | Blocked on failed native qualification; 17 scientific selectors and pivot operation/build measurements remain required with valid target conditions. |
| E5 | This assessment is Revise; qualification publication, finding reconciliation, enduring owners and safe retirement remain incomplete. |
| US01–US05 | Native mechanisms exist, but restart/ordinary outcome/codec/race/dependency obligations lack full acceptance; F01/F03/F04 are material to US01/US04, and F02 to selected preparation. |
| EF01 | Superseded bundle ownership replacement inspected; assembled retention/release acceptance remains incomplete. |
| EF02/EF08 | Actual owner reuse and compact consumers have positive source/focused controls; no newly measured performance claim. |
| EF03–EF07 | Scoped study/build/identity/generation mechanisms inspected; complete operation measurements and clean retirement/static acceptance pending. |
| 25/27 handoff | Physical closure, accuracy/response validity, branch/rank refusals, events/dynamics/fitting/continuation/partiality retained as obligations under 28e. Historical receipts do not close K3/K4/K5. |

### Accepted rule consequences, inspected separately

Operator acceptance of RC01–RC10 is retained; it does not establish implementation or ADR acceptance. These entries describe this dated review's coverage.

| Consequence | Evidence and completion limit |
|---|---|
| RC01: canonical store replacement | Canonical SurrealDB owners and displaced tracked production paths inspected; F01/F03 and E3 failures prevent acceptance. |
| RC02: automatic scientific retention | Ordinary success/reopen route exists; failed/partial/cancelled restart and backpressure terminal coverage remain incomplete. |
| RC03: declaration authority and storage lowering retirement | Registry-owned declarations and generated target lowering inspected; fresh generation/fixture acceptance remains required. |
| RC04: selected/native preparation and portable descriptions | Shared scientific kernels and reconstruction are implemented; F02 and valid deployed replay remain material gaps. |
| RC05: relevant identity versus full attestation | Distinct identity/attestation contracts inspected; no fixture receipt substitutes for current installed association. |
| RC06: consumer-shaped features | Removal and target scoping exist; no fresh complete powerset/native-lint acceptance was obtained. |
| RC07: scoped study claims | Database-native fenced claim and Rust policy owners exist; proper competing/killed-worker and lost-ack qualification remain open. |
| RC08: connected queries and Arrow boundary | Exact selection, copied Arrow and export mechanisms inspected; Python failures and missing eligible admission prevent closure. |
| RC09: unified protection/retention and clean rebuild | Protection/root protocols and clean rebuild direction inspected; lifecycle ownership fails F01, recovery remains condition-specific. |
| RC10: scoped build setup and output preservation | Build-input/producer tooling and retirement mechanisms inspected; comparable cache/build measurements and current generation acceptance are not established. |

### Original finding obligations

| Finding | Current audit evidence boundary |
|---|---|
| US01 | Canonical connected path exists; coherent restart/recovery and complete retired-path acceptance remain unqualified. |
| US02 | Automatic success retention works in sampled journeys; ordinary failure/partial/cancelled retention after restart remains open. |
| US03 | Exact codec/Arrow controls exist; no complete assembled protocol/scientific pass is claimed. |
| US04 | Guarded selection/claims and protection exist; F01, initialization completion and combined races prevent closure. |
| US05 | Selected sufficient admission and portable dependencies exist; F02, fixture closure and deployed compatibility remain unsettled. |
| EF01 | Superseded active-owner release is implemented; deliberate history and lifecycle retention must remain distinct, with F01 corrected. |
| EF02 | Native-owner reuse has positive source/focused evidence; this audit neither reopens its recorded local disposition nor promotes it to assembled acceptance. |
| EF03 | Scoped study discovery/claim exists; complete dispatch/preparation behavior and measurements remain required. |
| EF04 | Displaced dependencies/crates are removed in inspected owners; current full feature/build isolation acceptance remains incomplete. |
| EF05 | Relevant keys and outer attestation are separate; actual current artifact association remains required. |
| EF06 | Scoped native setup/build instrumentation exists; no comparable current build/cache result is established. |
| EF07 | Obsolete generation retirement and unchanged-output preservation mechanisms exist; stale recipe reference and missing current generation acceptance remain. |
| EF08 | Compact actual evaluator consumers have positive local evidence; no measured gain is inferred. |

### Cross-package scenarios that remain unsettled

Source supports lost-ack readback for run claims, batch append, closure and sealing. The study claim wrapper instead returns a driver error; its worker rereads scope and can rely on later assigned-attempt recovery. That is a material recovery uncertainty requiring an after-commit acknowledgment-loss control, not a confirmed duplicate-execution defect.

Existing tests cover cancelled queries, malformed later IPC blocks, exported incomplete staging, reader-versus-retirement acquisition and detached Arrow survival. They do not collectively establish live transport failure after emitted rows, ongoing multipage reads versus retirement, or source protection continuously covering analysis construction and final root admission under concurrency. Complete indexed responses before page exposure may be a valid realization; use actual transport evidence rather than requiring a particular SDK streaming method.

## Architectural and scientific assessment

| Foundation | Judgment in inspected scope |
|---|---|
| AP-01 | Satisfied: scientific interpretation, canonical operations, preparation, execution and boundary conversion have identifiable owners. |
| AP-02 | Satisfied locally: consumers use explicit scientific/runtime/store contracts; no second production storage abstraction was introduced. |
| AP-03 | Satisfied locally: runtime composes owners; persisted analyses reuse actual source/result contracts. |
| AP-04 | Adequate revision/run/product/lease model, but authoritative realization violated by lifecycle root reassignment. Full scientific model adequacy is not accepted by this audit. |
| AP-05 | Violated: F01 and F03 leave consequential constraints outside enforced admission. |
| AP-06 | Satisfied in sampled responsibility boundaries; failing assembled fixtures remain verification gaps rather than automatic architectural entanglement. |
| AP-07 | Violated: F02 introduces avoidable complete-preparation work. No whole-system performance/capacity acceptance. |

G1/G2/G4 have positive sampled authority/meaning/effect boundaries, with assembled scope unresolved. **G3/G5 fail** the root and initialization invariants. G6 has positive strict replay/compact transformation mechanisms but unresolved deployed equivalence. G7 cannot accept broad qualification claims beyond current receipts. G8 has no confirmed bespoke numerical substitute in inspected pivot scope; the mathematical libraries and native solvers retain their numerical machinery. **G9 fails**, without averaging the positive foundations against the violations.

| Scientific gate | Judgment |
|---|---|
| PS-G1 physical consistency | Unresolved for assembled acceptance: units/basis/reference/source identities survive the intended typed boundary, but current quantity/input failures and incomplete manifest acceptance prevent qualification. |
| PS-G2 well-posedness | Unresolved: shared scientific admission and structural owners remain; this audit does not establish every assembled branch/refusal/pre-solve journey. |
| PS-G3 numerical integrity | Unresolved: independent outcome/accuracy/derivative/domain checks remain required; source preservation and current campaign failures prevent a demonstrated end-to-end pass. |

The physical-semantics boundary stores identities and interpretation, not an alternate unit-conversion authority. Basis/reference conventions come from selected physical declarations; missingness and exact finite/diagnostic IEEE encodings are distinct. Stored primal/dual/sensitivity values require their original numerical conditions, not a graph edge or terminal label.

Numerical stages remain owned: selection/admission requires sufficient physical and structural premises; library-owned compilation/reconstruction requires guards, provider contracts and receipt compatibility; solve/response requires receiving-state bounds/domains/accuracy and declared derivative orders; retained completion requires independent original checks and truthful partiality. Persistence and export preserve those conclusions rather than minting scientific success.

## Verification and limits

**Conditions:** Linux local execution, initially clean commit above; pinned nightly 2026-09-29, Python 3.14.7, native libraries from the repository environment, released SurrealDB 3.3.0 with SDK 3.3.0. The isolated audit state used port 18428, authenticated loopback gRPC, RocksDB sync=every and a 4 MiB message limit. Its resource profile was 18 GiB total: 2 GiB server and two 8 GiB native worker slots. Native testing requested NEXTEST_TEST_THREADS=16 and inherited the recipe's memory cap and single-threaded numerical-library controls. No thresholds, cases or assertions were changed.

The Rust and Python commands were launched while the native graph was building. Python initially waited on Cargo's lock, then overlapped native execution. That differs from the intended serialized campaign: the observations are functional diagnostics under these actual conditions, not comparable performance measurements.

| Claim / command | Evidence and actual result, baseline zero |
|---|---|
| Isolated supervisor setup/start and `just canonical-init <audit-state>` | **Tested:** commands exit 0; normal native initialization verified interpretation. This does not exercise the incomplete-response branch in F03. |
| `just assessment build/audits/plan28-20261006-native --functional-scope native` with the state/thread settings above | **Tested, partial:** assessment exit 1; native leaf exit 100. Of 2,741 selected tests, Nextest reports 2,421 completed: 2,342 passed, 78 failed and one timed out. It also reports two interrupted tests and 320 not run. These cancellation statuses are not successful skips. |
| `just native-python build/audits/plan28-20261006-python` with the isolated state | **Tested, partial:** exit 2 after coordinator SIGINT. Pytest reports 156 passed and four failed out of 199 collected; the 1,000-point flash study was interrupted and 39 selected identities have no completed result. |
| `just assessment build/audits/plan28-20261006-static-quality --group quality` | **Tested:** exit 1, 15 gates attempted, nine passed, six failed; recorded source unchanged. |
| `just assessment build/audits/plan28-20261006-format --group fmt-check` | **Tested:** exit 1; Rust formatting and TOML formatting both fail, source unchanged. The TOML gate overlaps the quality report and is not a second unique failure. |
| `just doctor` after the environment-shaped import-linter failure | **Tested:** exit 0 for toolchain/environment checks; warning for an absent default canonical profile. The separate audit profile was present. This does not qualify producer association or resolve import-linter discovery. |
| `just docs` | **Tested:** initial exit 0, 253 chapters published/indexed; after adding this report, exit 0 with 254 chapters and the report's HTML present in the published site. Documentation publication does not establish product acceptance. |
| Audit server quiesce/stop | **Tested:** final `quiesce` and `stop --drained` exit 0; supervisor reports inactive and quiesced. Initial stop without the required drain acknowledgment refused, as designed. State and raw evidence were preserved. |

**Interruption and association limits:** during execution the working tree changed across over a hundred source/test/generated/tooling files. The coordinator sent SIGINT only to the two owned audit test processes, preserving concurrent work. The native version-5 report records source_unchanged=false, changed input paths, and selected/executed identity mismatch. Its terminal complete=true means reporting finished, not that all selected tests ran or qualification passed. The Python terminal receipt also reports identity/count errors: the interrupted XML contains an extra unnamed record, while the console reports 160 completed tests. Do not use either receipt as a positive functional prerequisite, and do not infer acceptance of the newer tree from the original binaries. Source findings are anchored to the reviewed commit; corrections in concurrent work require their own review/evidence.

Local raw evidence is retained under `build/audits/plan28-20261006-native/` (`checks.json`, `native-test.log`, selected identities, JUnit and native provenance), `build/audits/plan28-20261006-python/` (selection, imported-binary provenance, JUnit and terminal receipt), and the static-quality/format directories above. These ignored host artifacts are not committed release evidence.

The six quality failures are: 19 Python files would be formatted; 68 Python lint findings; import-linter could not locate package pse; TOML formatting in pse-diagnostics/Cargo.toml; typo findings including raw ELF probe tokens such as RELA; and agent lint referencing retired codegen-queries-check. Immutable scientific/probe evidence must not be rewritten indiscriminately to satisfy text lint. Static failures are reported against zero, with no baseline exemption and no claim they remain unchanged in concurrent edits.

Selected positive executed observations include `test_canonical_result_selection_and_progress_reopen`, `canonical_durable_constant_completion_reopens_exact_manifest`, concrete native implicit controls and canonical codec/reconstruction/retention controls. The Python public process and dynamic/fitting examples also completed. They establish the named sampled behavior under this run's conditions, not all terminal classes, larger studies, physical semantics or deployment eligibility.

**Not executed to a fresh stable target:** the proper `just worker-test` journey; `just canonical-recovery-test` process-kill/offline-restore control; complete reference manifest via `just modeling-conformance --manifest packages/reference/conformance.toml --report-dir <fresh-dir>`; `just parity`; full `just hygiene` and `just governance`; complete feature combinations/no-default and native/solver-contract lints. Some unit mechanisms were exercised within the partial native run, but these named campaigns cannot be substituted by that overlap. Missing valid producer prerequisites, observed failures and subsequent source drift prevented an assembled acceptance campaign. Running additional builds against an evolving tree would not settle the reviewed commit; a coordinated stable-source rerun is required. This is unresolved qualification scope, not evidence those unexecuted commands would fail.

**E4: blocked.** No scientific Criterion samples or pivot latency/build/cache campaign was launched, because its current zero-failure functional prerequisite was not met. All 17 required scientific selectors and applicable pivot measurements remain unmeasured here. **E5:** this bounded independent assessment is delivered with a Revise verdict; adopted-rule routing, complete qualification publication, finding closure and retirement remain unfinished.

No wheels, remote CI, other platforms, network-distributed server guarantees or power-loss durability are established. Process-kill/reopen controls cannot prove host power-loss safety. Correctness invocations use explicit force-validation and memory-capped native wrappers; measurement claims would need production-equivalent conditions separately.

Earlier governance/Clippy/powerset and Plan25/27 passes were not adopted simply because their date is recent. In particular, the inspected 20261006T062656 governance receipt runs now-retired PostgreSQL/query generators; it is predecessor evidence. The pre-pivot 2,823-test pass and 210-test Python pass retain their earlier source/environment scope. Three accuracy smoke controls explicitly say measured:false. A bounded failure-only audit cannot turn those into current target acceptance.

## Disposition and closure order

**Rule impacts:** None for F01–F04’s corrective directions: they implement the already intended ownership, truthful completion and contextual-accuracy contracts. R0’s existing authorized RC01–RC10 and proposed ADR adoption remain at their owners; this report applies no rule change.

1. Enforce lifecycle root ownership and initialization completion; preserve real run/attempt/source semantics.
2. Complete target input/fixture and runner prerequisites; associate actual current runtime/worker/installed Python producer captures.
3. Remove repeated selected-preparation work within exact guarded admission; retain source, shadowing, provider and numerical checks.
4. Re-run affected targeted controls, then the assembled E3 scientific, lifecycle, recovery, Python, manifest, parity and static scope against zero. Diagnose numerical premises before changing any tolerance.
5. Only after functional qualification, execute/report valid E4 scientific and pivot measurements; then reconcile existing findings, publish demonstrated enduring scope and complete the decision/retirement route.

No source correction, tolerance change, plan-status change, ADR acceptance, retirement, commit or push was performed by this audit.
